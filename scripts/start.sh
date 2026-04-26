#!/usr/bin/env bash
# =============================================================================
# Nexus - 启动所有服务
# 用法: ./scripts/start.sh [--no-frontend] [--build]
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PID_DIR="${PROJECT_ROOT}/.pids"

# ── 兼容 Linux/macOS ──────────────────────────────────────────────────────────

# ss -tlnp 的 Linux 实现；macOS 用 lsof
_ss_listen_port() {
    local port="$1"
    if command_exists ss; then
        ss -tlnp "sport = :${port}" 2>/dev/null | awk 'NR>1 {print $NF}' | sed 's/.*pid=//' | grep -oE '[0-9]+' | head -1
    elif command_exists lsof; then
        lsof -ti ":${port}" 2>/dev/null | head -1
    fi
}

# hostname -I 的 Linux 实现；macOS 用 ifconfig
_get_local_ips() {
    if command_exists hostname && hostname -I &>/dev/null; then
        hostname -I 2>/dev/null | tr ' ' '\n' | grep -E '^[0-9]+\.'
    elif command_exists ifconfig; then
        ifconfig 2>/dev/null | grep 'inet ' | awk '{print $2}' | grep -v '^127\.'
    fi
}

# 检测当前系统类型
_detect_os_family() {
    if [[ "$(uname)" == "Darwin" ]]; then
        OS_FAMILY="macos"
    elif [[ -f /etc/os-release ]]; then
        . /etc/os-release
        case "${ID:-}" in
            ubuntu|debian|linuxmint|pop)    OS_FAMILY="debian" ;;
            centos|rhel|rocky|almalinux|ol) OS_FAMILY="rhel" ;;
            fedora)  OS_FAMILY="fedora" ;;
            arch*)   OS_FAMILY="arch" ;;
            *)       OS_FAMILY="unknown" ;;
        esac
    else
        OS_FAMILY="unknown"
    fi
}

# ── 公共函数 ───────────────────────────────────────────────────────────────────

if [[ -t 1 ]]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'
    BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'
    DIM='\033[2m'; RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' CYAN='' BOLD='' DIM='' RESET=''
fi

log_info()    { echo -e "${DIM}[$(date '+%H:%M:%S')]${RESET} ${BLUE}ℹ${RESET}  $*"; }
log_ok()      { echo -e "${DIM}[$(date '+%H:%M:%S')]${RESET} ${GREEN}✓${RESET}  $*"; }
log_warn()    { echo -e "${DIM}[$(date '+%H:%M:%S')]${RESET} ${YELLOW}⚠${RESET}  $*"; }
log_step()    { echo ""; echo -e "${BOLD}═══ $* ═══${RESET}"; }
log_cmd()     { echo -ne "${DIM}[$(date '+%H:%M:%S')]${RESET} ${CYAN}→${RESET}  $*..."; }
log_cmd_ok()  { echo -e "\r${DIM}[$(date '+%H:%M:%S')]${RESET} ${GREEN}✓${RESET}  $*"; }
log_cmd_fail(){ echo -e "\r${DIM}[$(date '+%H:%M:%S')]${RESET} ${RED}✗${RESET}  $*" >&2; }

banner() {
    echo ""
    echo -e "${BOLD}${CYAN}╔══════════════════════════════════════════════╗${RESET}"
    printf "${BOLD}${CYAN}║${RESET}  %-42s  ${BOLD}${CYAN}║${RESET}\n" "${1:-Nexus}"
    echo -e "${BOLD}${CYAN}╚══════════════════════════════════════════════╝${RESET}"
    echo ""
}

# ── 参数解析 ─────────────────────────────────────────────────────────────────

NO_FRONTEND=false
SHOULD_BUILD=false

for arg in "$@"; do
    case "$arg" in
        --no-frontend) NO_FRONTEND=true ;;
        --build)       SHOULD_BUILD=true ;;
        --help|-h)
            echo "用法: $0 [--no-frontend] [--build]"
            echo "  --no-frontend  不启动前端 dev server"
            echo "  --build        启动前重新编译后端"
            exit 0
            ;;
    esac
done

# ── 清理 ─────────────────────────────────────────────────────────────────────

cleanup() {
    echo ""
    log_info "正在停止服务..."
    "${SCRIPT_DIR}/stop.sh" 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

# ── 清理旧进程 ───────────────────────────────────────────────────────────────

cleanup_old_processes() {
    # 清理之前残留的 start.sh 进程（排除自身）
    local self_pid=$$
    local old_pids
    old_pids=$(ps aux | grep -E "scripts/start\.sh" | grep -v grep | awk '{print $2}' 2>/dev/null || true)
    for pid in $old_pids; do
        [[ "$pid" == "$self_pid" ]] && continue
        kill -9 "$pid" 2>/dev/null || true
    done
}

# ── 加载配置 ─────────────────────────────────────────────────────────────────

mkdir -p "$PID_DIR"

load_env() {
    if [[ -f "${PROJECT_ROOT}/.env.local" ]]; then
        # shellcheck source=/dev/null
        source "${PROJECT_ROOT}/.env.local"
        log_info "已加载 .env.local"
    elif [[ -f "${PROJECT_ROOT}/.env" ]]; then
        # shellcheck source=/dev/null
        source "${PROJECT_ROOT}/.env"
        log_info "已加载 .env"
    fi
}

# ── 端口检查 ─────────────────────────────────────────────────────────────────

check_and_free_port() {
    local port="$1"
    local pid
    pid=$(_ss_listen_port "$port" || true)

    if [[ -n "$pid" ]]; then
        log_warn "端口 ${port} 已被进程 (PID ${pid}) 占用，正在清理..."
        kill "$pid" 2>/dev/null || true
        local retries=10
        while kill -0 "$pid" 2>/dev/null && (( retries-- > 0 )); do sleep 1; done
        if kill -0 "$pid" 2>/dev/null; then
            kill -9 "$pid" 2>/dev/null || true
            log_warn "已强制终止占用端口 ${port} 的进程"
        else
            log_ok "已释放端口 ${port}"
        fi
    fi
}

# ── 启动后端 ─────────────────────────────────────────────────────────────────

start_backend() {
    log_step "启动后端服务"

    # 确保 cargo 在 PATH 中
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    elif [[ -d "$HOME/.cargo/bin" ]] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    if ! command -v cargo &>/dev/null; then
        log_cmd_fail "找不到 cargo，请先安装 Rust"; exit 1
    fi

    if [[ "$SHOULD_BUILD" == "true" ]]; then
        log_cmd "重新编译后端"
        (cd "${PROJECT_ROOT}/service" && cargo build --release --package nexus-service 2>&1 | tail -3)
        log_cmd_ok "编译完成"
    fi

    # 确保数据库和 Redis 运行
    if ! pg_isready &>/dev/null; then
        log_warn "PostgreSQL 未运行，尝试启动..."
        if [[ "$OS_FAMILY" == "macos" ]]; then
            brew services start postgresql@16 2>/dev/null || brew services start postgresql 2>/dev/null || true
        else
            sudo systemctl start postgresql 2>/dev/null || sudo systemctl start postgresql-16 2>/dev/null || sudo service postgresql start 2>/dev/null || true
        fi
        sleep 2
    fi
    if ! redis-cli ping &>/dev/null 2>&1; then
        log_warn "Redis 未运行，尝试启动..."
        if [[ "$OS_FAMILY" == "macos" ]]; then
            brew services start redis 2>/dev/null || true
        else
            sudo systemctl start redis-server 2>/dev/null || sudo systemctl start redis 2>/dev/null || true
        fi
        sleep 2
    fi

    # 自动检测本机 IP，添加到 CORS 允许列表
    local host_ips
    host_ips=$(_get_local_ips | head -5 || true)
    local cors_origins="${CORS_ALLOWED_ORIGINS:-http://localhost:3000,http://localhost:3001}"
    for ip in $host_ips; do
        for p in 3000 3001; do
            cors_origins="${cors_origins},http://${ip}:${p}"
        done
    done

    log_cmd "启动 nexus-service (port ${PORT:-8080})"

    cd "${PROJECT_ROOT}/service"
    nohup env \
        RUST_LOG="${RUST_LOG:-info}" \
        DATABASE_URL="${DATABASE_URL:-postgres://nexus:nexus_dev_password@localhost:5432/nexus}" \
        REDIS_URL="${REDIS_URL:-redis://localhost:6379}" \
        JWT_SECRET="${JWT_SECRET:-change-me}" \
        PORT="${PORT:-8080}" \
        MIGRATIONS_PATH="${MIGRATIONS_PATH:-./db/migrations}" \
        CORS_ALLOWED_ORIGINS="${cors_origins}" \
        cargo run --release --package nexus-service \
        > "${PID_DIR}/nexus-service.log" 2>&1 &
    disown
    echo $! > "${PID_DIR}/nexus-service.pid"

    # 等待后端就绪
    local retries=10
    while ! (curl -sf "http://localhost:${PORT:-8080}/health" >/dev/null 2>&1) && (( retries-- > 0 )); do
        sleep 1
    done

    if (curl -sf "http://localhost:${PORT:-8080}/health" >/dev/null 2>&1); then
        log_cmd_ok "后端已启动 (PID $(cat "${PID_DIR}/nexus-service.pid"))"
    else
        log_cmd_fail "后端启动超时，请检查日志: ${PID_DIR}/nexus-service.log"
        echo ""
        log_info "最近日志:"
        tail -10 "${PID_DIR}/nexus-service.log" 2>/dev/null | while IFS= read -r line; do
            echo -e "  ${DIM}${line}${RESET}"
        done
        echo ""
        exit 1
    fi
}

# ── 启动前端 ─────────────────────────────────────────────────────────────────

start_frontend_admin() {
    log_cmd "启动 admin 面板 (port 3000)"
    cd "${PROJECT_ROOT}/app/admin"
    nohup npm run dev -- --port 3000 --host > "${PID_DIR}/nexus-admin.log" 2>&1 &
    disown
    echo $! > "${PID_DIR}/nexus-admin.pid"
    log_cmd_ok "admin 面板已启动 (PID $(cat "${PID_DIR}/nexus-admin.pid"))"
}

start_frontend_client() {
    log_cmd "启动 client 客户端 (port 3001)"
    cd "${PROJECT_ROOT}/app/client"
    nohup npm run dev -- --port 3001 --host > "${PID_DIR}/nexus-client.log" 2>&1 &
    disown
    echo $! > "${PID_DIR}/nexus-client.pid"
    log_cmd_ok "client 客户端已启动 (PID $(cat "${PID_DIR}/nexus-client.pid"))"
}

# ── 主流程 ───────────────────────────────────────────────────────────────────

main() {
    banner "Nexus Start"
    cd "$PROJECT_ROOT"
    _detect_os_family
    load_env

    cleanup_old_processes

    check_and_free_port "${PORT:-8080}"
    start_backend

    if [[ "$NO_FRONTEND" == "false" ]]; then
        check_and_free_port 3000
        check_and_free_port 3001
        log_step "启动前端服务"
        start_frontend_admin
        start_frontend_client
    fi

    echo ""
    log_ok "${BOLD}所有服务已启动${RESET}"
    echo ""
    echo -e "  API:    ${CYAN}http://localhost:${PORT:-8080}${RESET}"
    if [[ "$NO_FRONTEND" == "false" ]]; then
        echo -e "  Admin:  ${CYAN}http://localhost:3000${RESET}"
        echo -e "  Client: ${CYAN}http://localhost:3001${RESET}"
    fi
    echo ""
    echo -e "  ${DIM}日志: ${PID_DIR}/*.log${RESET}"
    echo -e "  ${DIM}停止: Ctrl+C 或 ./scripts/stop.sh${RESET}"
    echo ""

    # 保持脚本运行，等待信号（sleep & wait 可被信号中断）
    while :; do sleep 86400 & wait $!; done
}

main "$@"
