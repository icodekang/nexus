#!/usr/bin/env bash
# =============================================================================
# Nexus - 停止所有服务
# 用法: ./scripts/stop.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PID_DIR="${PROJECT_ROOT}/.pids"

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
log_cmd()     { echo -ne "${DIM}[$(date '+%H:%M:%S')]${RESET} ${CYAN}→${RESET}  $*..."; }
log_cmd_ok()  { echo -e "\r${DIM}[$(date '+%H:%M:%S')]${RESET} ${GREEN}✓${RESET}  $*"; }
log_step()    { echo ""; echo -e "${BOLD}═══ $* ═══${RESET}"; }

# ── 停止单个服务 ─────────────────────────────────────────────────────────────

stop_service() {
    local name="$1"
    local pid_file="${PID_DIR}/${name}.pid"

    if [[ ! -f "$pid_file" ]]; then
        return 0
    fi

    local pid
    pid=$(cat "$pid_file" 2>/dev/null || echo "")

    if [[ -z "$pid" ]]; then
        rm -f "$pid_file"
        return 0
    fi

    if kill -0 "$pid" 2>/dev/null; then
        log_cmd "停止 ${name} (PID ${pid})"
        kill "$pid" 2>/dev/null || true
        # 同时杀掉子进程
        pkill -P "$pid" 2>/dev/null || true
        local retries=10
        while kill -0 "$pid" 2>/dev/null && (( retries-- > 0 )); do
            sleep 1
        done
        if kill -0 "$pid" 2>/dev/null; then
            kill -9 "$pid" 2>/dev/null || true
            pkill -9 -P "$pid" 2>/dev/null || true
            log_cmd_ok "${name} 已强制停止"
        else
            log_cmd_ok "${name} 已停止"
        fi
    else
        log_info "${name} 进程已不存在 (PID ${pid})"
    fi

    rm -f "$pid_file"
}

# ── 主流程 ───────────────────────────────────────────────────────────────────

main() {
    if [[ ! -d "$PID_DIR" ]]; then
        log_info "没有找到运行中的服务"
        exit 0
    fi

    log_step "停止 Nexus 服务"

    stop_service "nexus-client"
    stop_service "nexus-admin"
    stop_service "nexus-service"

    # 兜底：清理可能残留的端口占用
    for port in 8080 3000 3001; do
        local pid
        pid=$(ss -tlnp "sport = :${port}" 2>/dev/null | awk 'NR>1 {print $NF}' | grep -oP 'pid=\K[0-9]+' | head -1 || true)
        if [[ -n "$pid" ]]; then
            log_warn "端口 ${port} 仍有进程 (PID ${pid})，强制清理"
            kill -9 "$pid" 2>/dev/null || true
            pkill -9 -P "$pid" 2>/dev/null || true
        fi
    done

    rmdir "$PID_DIR" 2>/dev/null || true

    echo ""
    log_ok "所有服务已停止"
}

main "$@"
