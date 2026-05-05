#!/usr/bin/env bash
# =============================================================================
# Nexus - 环境准备（环境检测 → 安装依赖 → 生成配置 → 初始化数据库 → 编译）
# 用法: ./scripts/setup.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── 公共函数 ───────────────────────────────────────────────────────────────────

if [[ -t 1 ]]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'
    BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'
    DIM='\033[2m'; RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' CYAN='' BOLD='' DIM='' RESET=''
fi

_NEXUS_START_TS=$(date +%s)

_nexus_ts()    { date '+%H:%M:%S'; }
_nexus_elapsed() {
    local diff=$(( $(date +%s) - _NEXUS_START_TS ))
    local m=$(( diff / 60 )) s=$(( diff % 60 ))
    (( m > 0 )) && echo "${m}m ${s}s" || echo "${s}s"
}

log_info()    { echo -e "${DIM}[$(_nexus_ts)]${RESET} ${BLUE}ℹ${RESET}  $*"; }
log_ok()      { echo -e "${DIM}[$(_nexus_ts)]${RESET} ${GREEN}✓${RESET}  $*"; }
log_warn()    { echo -e "${DIM}[$(_nexus_ts)]${RESET} ${YELLOW}⚠${RESET}  $*"; }
log_error()   { echo -e "${DIM}[$(_nexus_ts)]${RESET} ${RED}✗${RESET}  $*" >&2; }
log_step()    { echo ""; echo -e "${BOLD}═══ $* ═══${RESET}"; }
log_cmd()     { echo -ne "${DIM}[$(_nexus_ts)]${RESET} ${CYAN}→${RESET}  $*..."; }
log_cmd_ok()  { echo -e "\r${DIM}[$(_nexus_ts)]${RESET} ${GREEN}✓${RESET}  $*"; }
log_cmd_fail(){ echo -e "\r${DIM}[$(_nexus_ts)]${RESET} ${RED}✗${RESET}  $*" >&2; }

banner() {
    echo ""
    echo -e "${BOLD}${CYAN}╔══════════════════════════════════════════════╗${RESET}"
    printf "${BOLD}${CYAN}║${RESET}  %-42s  ${BOLD}${CYAN}║${RESET}\n" "${1:-Nexus}"
    echo -e "${BOLD}${CYAN}╚══════════════════════════════════════════════╝${RESET}"
    echo ""
}

command_exists() { command -v "$1" &>/dev/null; }
version_gte()    { printf '%s\n%s' "$1" "$2" | sort -rV -C; }
is_project_root(){ [[ -f "docker-compose.yml" && -d "service" && -d "app" ]]; }

ensure_project_root() {
    if ! is_project_root; then
        log_error "请在项目根目录下运行此脚本"
        exit 1
    fi
}

# ── 环境检测 ───────────────────────────────────────────────────────────────────

detect_os() {
    log_info "检测运行环境..."
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64" ;;
        aarch64|arm64) ARCH="arm64" ;;
    esac

    if [[ "$(uname)" == "Darwin" ]]; then
        OS_FAMILY="macos"; OS_NAME="macOS"
        OS_VERSION=$(sw_vers -productVersion 2>/dev/null || echo "unknown")
        PKG_MGR="brew"
    elif [[ -f /etc/os-release ]]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        OS_NAME="${NAME:-unknown}"; OS_VERSION="${VERSION_ID:-unknown}"
        case "${ID:-}" in
            ubuntu|debian|linuxmint|pop)    OS_FAMILY="debian"; PKG_MGR="apt-get" ;;
            centos|rhel|rocky|almalinux|ol) OS_FAMILY="rhel";   PKG_MGR="$(command_exists dnf && echo dnf || echo yum)" ;;
            fedora)  OS_FAMILY="fedora"; PKG_MGR="dnf" ;;
            arch*)   OS_FAMILY="arch";  PKG_MGR="pacman" ;;
            *)       OS_FAMILY="unknown"; PKG_MGR="" ;;
        esac
    else
        OS_FAMILY="unknown"; OS_NAME="unknown"; OS_VERSION="unknown"; PKG_MGR=""
    fi
    log_ok "操作系统: ${OS_NAME} ${OS_VERSION} (${ARCH})"

    # WSL: strip Windows-injected /mnt/ paths to prevent Windows binaries
    # (e.g. Windows Node.js) from interfering with WSL-native tool detection
    if grep -qi microsoft /proc/version 2>/dev/null || grep -qi wsl /proc/version 2>/dev/null; then
        IS_WSL=true
        local new_path=""
        IFS=':' read -ra path_parts <<< "$PATH"
        for part in "${path_parts[@]}"; do
            [[ "$part" != /mnt/* ]] && new_path="${new_path}:${part}"
        done
        export PATH="${new_path#:}"
        log_ok "WSL 环境: 已过滤 Windows 路径"
    fi

    # Source nvm if installed (for per-user Node.js in WSL)
    if [[ -s "$HOME/.nvm/nvm.sh" ]]; then
        # shellcheck source=/dev/null
        . "$HOME/.nvm/nvm.sh" 2>/dev/null || true
    fi
}

detect_rust() {
    HAS_RUST=false
    if command_exists rustc; then
        HAS_RUST=true
        RUST_VERSION=$(rustc --version | awk '{print $2}')
        log_ok "Rust: rustc ${RUST_VERSION} (已安装)"
    else
        log_warn "Rust: 未安装"
    fi
}

detect_node() {
    HAS_NODE=false
    if command_exists node; then
        NODE_VERSION=$(node --version | sed 's/^v//')
        if version_gte "$NODE_VERSION" "18.0.0"; then
            HAS_NODE=true
            log_ok "Node.js: v${NODE_VERSION} (已安装)"
        else
            log_warn "Node.js: v${NODE_VERSION} (需要 >= 18)"
        fi
    else
        log_warn "Node.js: 未安装"
    fi
}

detect_postgres() {
    HAS_PG=false
    if command_exists psql; then
        HAS_PG=true
        PG_VERSION=$(psql --version | awk '{print $3}')
        log_ok "PostgreSQL: ${PG_VERSION} (已安装)"
    else
        log_warn "PostgreSQL: 未安装"
    fi
}

detect_redis() {
    HAS_REDIS=false
    if command_exists redis-cli; then
        HAS_REDIS=true
        REDIS_VERSION=$(redis-cli --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        log_ok "Redis: ${REDIS_VERSION} (已安装)"
    else
        log_warn "Redis: 未安装"
    fi
}

run_detect() {
    detect_os; detect_rust; detect_node; detect_postgres; detect_redis
}

# ── 安装依赖 ───────────────────────────────────────────────────────────────────

_pkg_update() {
    case "$PKG_MGR" in
        apt-get) sudo apt-get update -qq ;;
        yum)     sudo yum makecache -q ;;
        dnf)     sudo dnf makecache -q ;;
        pacman)  sudo pacman -Sy --noconfirm ;;
        brew)    brew update ;;
    esac
}

_pkg_install() {
    case "$PKG_MGR" in
        apt-get) sudo apt-get install -y -qq "$@" ;;
        yum)     sudo yum install -y -q "$@" ;;
        dnf)     sudo dnf install -y -q "$@" ;;
        pacman)  sudo pacman -S --noconfirm "$@" ;;
        brew)    brew install --quiet "$@" 2>/dev/null ;;
    esac
}

install_build_deps() {
    log_cmd "安装构建依赖 (pkg-config, libssl-dev)"
    case "$OS_FAMILY" in
        debian)    _pkg_install build-essential pkg-config libssl-dev curl lsb-release gnupg ;;
        rhel|fedora) _pkg_install gcc gcc-c++ make pkgconfig openssl-devel curl ;;
        macos)     xcode-select --install 2>/dev/null || true; _pkg_install pkg-config openssl ;;
        arch)      _pkg_install base-devel pkg-config openssl curl ;;
    esac
    log_cmd_ok "构建依赖安装完成"
}

install_rust() {
    if [[ "$HAS_RUST" == "true" ]]; then
        log_info "Rust 已安装，跳过"; return 0
    fi
    log_cmd "安装 Rust 工具链"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable 2>&1 | tail -3
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
    if command_exists rustc; then
        HAS_RUST=true; RUST_VERSION=$(rustc --version | awk '{print $2}')
        log_cmd_ok "Rust rustc ${RUST_VERSION} 安装完成"
    else
        log_cmd_fail "Rust 安装失败"; exit 1
    fi
}

install_node() {
    if [[ "$HAS_NODE" == "true" ]]; then
        log_info "Node.js 已安装，跳过"; return 0
    fi
    log_cmd "安装 Node.js 20.x"

    # WSL: use nvm (no sudo required, avoids Windows Node.js conflicts)
    if [[ "${IS_WSL:-false}" == "true" ]]; then
        _install_node_via_nvm
    else
        case "$OS_FAMILY" in
            debian)    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - 2>&1 | tail -1; _pkg_install nodejs ;;
            rhel|fedora) curl -fsSL https://rpm.nodesource.com/setup_20.x | sudo bash - 2>&1 | tail -1; _pkg_install nodejs ;;
            macos)     _pkg_install node ;;
            arch)      _pkg_install nodejs npm ;;
            *)         log_cmd_fail "不支持的系统: ${OS_FAMILY}"; exit 1 ;;
        esac
    fi

    if command_exists node; then
        HAS_NODE=true; NODE_VERSION=$(node --version | sed 's/^v//')
        log_cmd_ok "Node.js v${NODE_VERSION} 安装完成"
    else
        log_cmd_fail "Node.js 安装失败"; exit 1
    fi
}

# Install Node.js via direct binary download (no sudo, no git required)
_install_node_via_nvm() {
    local node_version="20.19.5"
    local node_dir="$HOME/.local/node-v${node_version}-linux-x64"
    local node_bin="${node_dir}/bin"

    if [[ -x "${node_bin}/node" ]]; then
        export PATH="${node_bin}:$PATH"
        return 0
    fi

    log_info "下载 Node.js v${node_version} 预编译包..."
    mkdir -p "$node_dir"
    local tarball="/tmp/node-v${node_version}.tar.xz"
    local download_urls=(
        "https://npmmirror.com/mirrors/node/v${node_version}/node-v${node_version}-linux-x64.tar.xz"
        "https://nodejs.org/dist/v${node_version}/node-v${node_version}-linux-x64.tar.xz"
    )
    local downloaded=false
    for url in "${download_urls[@]}"; do
        if curl -fsSL --connect-timeout 30 -o "$tarball" "$url" 2>/dev/null; then
            downloaded=true; break
        fi
    done
    if [[ "$downloaded" != "true" ]]; then
        log_cmd_fail "Node.js 下载失败（请检查网络）"; exit 1
    fi
    tar -xJf "$tarball" -C "$HOME/.local/" 2>/dev/null
    rm -f "$tarball"

    if [[ ! -x "${node_bin}/node" ]]; then
        log_cmd_fail "Node.js 解压失败"; exit 1
    fi
    export PATH="${node_bin}:$PATH"

    # Persist PATH in .bashrc for future sessions
    local bashrc="$HOME/.bashrc"
    local path_line="export PATH=\"${node_bin}:\$PATH\""
    if ! grep -qF "${node_bin}" "$bashrc" 2>/dev/null; then
        echo "" >> "$bashrc"
        echo "# Node.js (nexus setup)" >> "$bashrc"
        echo "$path_line" >> "$bashrc"
    fi
}

install_postgres() {
    if [[ "$HAS_PG" == "true" ]]; then
        log_info "PostgreSQL 已安装，跳过"; return 0
    fi
    log_cmd "安装 PostgreSQL 16"
    case "$OS_FAMILY" in
        debian)
            sudo sh -c 'echo "deb https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
            curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo gpg --dearmor -o /usr/share/keyrings/postgresql-archive-keyring.gpg 2>/dev/null
            _pkg_update; _pkg_install postgresql-16 postgresql-client-16
            ;;
        rhel|fedora)
            _pkg_install postgresql16-server postgresql16
            sudo /usr/pgsql-16/bin/postgresql-16-setup initdb 2>/dev/null || true
            ;;
        macos) _pkg_install postgresql@16; brew link --force postgresql@16 2>/dev/null || true ;;
        arch)  _pkg_install postgresql; sudo -u postgres initdb -D /var/lib/postgres/data 2>/dev/null || true ;;
        *)     log_cmd_fail "不支持的系统: ${OS_FAMILY}"; exit 1 ;;
    esac
    if command_exists psql; then
        HAS_PG=true; PG_VERSION=$(psql --version | awk '{print $3}')
        log_cmd_ok "PostgreSQL ${PG_VERSION} 安装完成"
    else
        log_cmd_fail "PostgreSQL 安装失败"; exit 1
    fi
}

start_postgres() {
    pg_isready &>/dev/null && return 0
    log_cmd "启动 PostgreSQL 服务"
    if [[ "$OS_FAMILY" == "macos" ]]; then
        brew services start postgresql@16 2>/dev/null || brew services start postgresql 2>/dev/null || true
    else
        sudo systemctl start postgresql 2>/dev/null || sudo systemctl start postgresql-16 2>/dev/null || sudo service postgresql start 2>/dev/null || true
        sudo systemctl enable postgresql 2>/dev/null || sudo systemctl enable postgresql-16 2>/dev/null || true
    fi
    local retries=10
    while ! pg_isready &>/dev/null && (( retries-- > 0 )); do sleep 1; done
    if pg_isready &>/dev/null; then
        log_cmd_ok "PostgreSQL 已启动"
    else
        log_cmd_fail "PostgreSQL 启动失败"; exit 1
    fi
}

install_redis() {
    if [[ "$HAS_REDIS" == "true" ]]; then
        log_info "Redis 已安装，跳过"; return 0
    fi
    log_cmd "安装 Redis"
    case "$OS_FAMILY" in
        debian|rhel|fedora|macos|arch) _pkg_install redis-server 2>/dev/null || _pkg_install redis ;;
        *)     log_cmd_fail "不支持的系统: ${OS_FAMILY}"; exit 1 ;;
    esac
    if command_exists redis-cli; then
        HAS_REDIS=true; log_cmd_ok "Redis 安装完成"
    else
        log_cmd_fail "Redis 安装失败"; exit 1
    fi
}

start_redis() {
    redis-cli ping &>/dev/null 2>&1 && return 0
    log_cmd "启动 Redis 服务"
    if [[ "$OS_FAMILY" == "macos" ]]; then
        brew services start redis 2>/dev/null || true
    else
        sudo systemctl start redis-server 2>/dev/null || sudo systemctl start redis 2>/dev/null || true
        sudo systemctl enable redis-server 2>/dev/null || sudo systemctl enable redis 2>/dev/null || true
    fi
    local retries=10
    while ! redis-cli ping &>/dev/null 2>&1 && (( retries-- > 0 )); do sleep 1; done
    if redis-cli ping &>/dev/null 2>&1; then
        log_cmd_ok "Redis 已启动"
    else
        log_cmd_fail "Redis 启动失败"; exit 1
    fi
}

install_all_deps() {
    log_step "安装系统依赖"
    install_build_deps; install_rust; install_node; install_postgres; install_redis
    start_postgres; start_redis
}

# ── 生成配置 ───────────────────────────────────────────────────────────────────

init_config() {
    log_step "初始化配置"
    local config="${PROJECT_ROOT}/.env.local"

    if [[ -f "$config" ]]; then
        log_info "配置文件 .env.local 已存在"
        local needs_update=false
        for field in DATABASE_URL REDIS_URL JWT_SECRET; do
            grep -q "^${field}=" "$config" || needs_update=true
        done
        if [[ "$needs_update" == "true" ]]; then
            log_cmd "补充缺失配置字段"
            _append_missing_fields "$config"
            log_cmd_ok "配置已补充"
        else
            log_cmd_ok "配置文件完整，无需修改"
        fi
    else
        log_cmd "生成 .env.local"
        _generate_config "$config"
        log_cmd_ok "配置文件已生成"
    fi
    log_info "配置文件: ${BOLD}.env.local${RESET}"
}

_generate_config() {
    cat > "$1" << EOF
# Nexus 本地开发环境配置
# 自动生成于 $(date)

DATABASE_URL=postgres://nexus:nexus_dev_password@localhost:5432/nexus
REDIS_URL=redis://localhost:6379
PORT=8080
JWT_SECRET=$(openssl rand -hex 32)
MIGRATIONS_PATH=./db/migrations
RUST_LOG=info
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:3001

# API Keys（按需填写）
OPENAI_API_KEY=
ANTHROPIC_API_KEY=
GOOGLE_API_KEY=
DEEPSEEK_API_KEY=
CUSTOM_PROVIDERS=
EOF
}

_append_missing_fields() {
    local config="$1"
    declare -A defaults=(
        [DATABASE_URL]="postgres://nexus:nexus_dev_password@localhost:5432/nexus"
        [REDIS_URL]="redis://localhost:6379"
        [PORT]="8080"
        [JWT_SECRET]="$(openssl rand -hex 32)"
        [MIGRATIONS_PATH]="./db/migrations"
        [RUST_LOG]="info"
        [CORS_ALLOWED_ORIGINS]="http://localhost:3000,http://localhost:3001"
    )
    for key in "${!defaults[@]}"; do
        if ! grep -q "^${key}=" "$config"; then
            echo "${key}=${defaults[$key]}" >> "$config"
        fi
    done
}

# postgres 连接方式：Linux 用 sudo -u postgres，macOS 用 brew 路径或直接连接
_pg_cmd() {
    # 如果没有指定 -d，默认连接 postgres 数据库
    local auto_db=""
    if [[ "$*" != *"-d "* && "$*" != *"--dbname"* ]]; then
        auto_db="-d postgres"
    fi
    if [[ "$OS_FAMILY" == "macos" ]]; then
        local psql_bin="psql"
        if [[ -x "/opt/homebrew/bin/psql" ]]; then
            psql_bin="/opt/homebrew/bin/psql"
        elif [[ -x "/usr/local/bin/psql" ]]; then
            psql_bin="/usr/local/bin/psql"
        fi
        $psql_bin $auto_db "$@"
    else
        sudo -u postgres psql $auto_db "$@"
    fi
}

# ── 初始化数据库 ───────────────────────────────────────────────────────────────

setup_database() {
    log_step "初始化数据库"

    if ! pg_isready &>/dev/null; then
        log_error "PostgreSQL 未运行，请先启动 PostgreSQL"; exit 1
    fi

    # 创建用户
    log_cmd "检查数据库用户 nexus"
    if _pg_cmd -tAc "SELECT 1 FROM pg_roles WHERE rolname='nexus'" 2>/dev/null | grep -q 1; then
        log_cmd_ok "用户 nexus 已存在"
    else
        _pg_cmd -c "CREATE USER nexus WITH PASSWORD 'nexus_dev_password' CREATEDB;" 2>&1 >/dev/null
        log_cmd_ok "用户 nexus 创建完成"
    fi

    # 创建数据库
    log_cmd "检查数据库 nexus"
    if _pg_cmd -tAc "SELECT 1 FROM pg_database WHERE datname='nexus'" 2>/dev/null | grep -q 1; then
        log_cmd_ok "数据库 nexus 已存在"
    else
        _pg_cmd -c "CREATE DATABASE nexus OWNER nexus;" 2>&1 >/dev/null
        log_cmd_ok "数据库 nexus 创建完成"
    fi
    _pg_cmd -c "GRANT ALL PRIVILEGES ON DATABASE nexus TO nexus;" 2>/dev/null || true
    _pg_cmd -d nexus -c "GRANT ALL ON SCHEMA public TO nexus;" 2>/dev/null || true

    # 迁移跟踪表
    PGPASSWORD=nexus_dev_password psql -U nexus -d nexus -h localhost -c "
        CREATE TABLE IF NOT EXISTS _nexus_migrations (
            filename VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
    " 2>/dev/null || true

    # 执行迁移
    local migrations_dir="service/db/migrations"
    if [[ -d "$migrations_dir" ]]; then
        local total=0 applied=0 skipped=0
        for f in $(find "$migrations_dir" -name "*.sql" | sort); do
            local fn; fn=$(basename "$f")
            total=$(( total + 1 ))
            local done
            done=$(PGPASSWORD=nexus_dev_password psql -U nexus -d nexus -h localhost -tAc \
                "SELECT 1 FROM _nexus_migrations WHERE filename='${fn}';" 2>/dev/null || echo "")
            if [[ "$done" == "1" ]]; then
                skipped=$(( skipped + 1 )); continue
            fi
            log_cmd "执行迁移 ${fn}"
            if PGPASSWORD=nexus_dev_password psql -U nexus -d nexus -h localhost -f "$f" 2>&1 >/dev/null; then
                PGPASSWORD=nexus_dev_password psql -U nexus -d nexus -h localhost -c \
                    "INSERT INTO _nexus_migrations (filename) VALUES ('${fn}');" 2>/dev/null || true
                applied=$(( applied + 1 ))
                log_cmd_ok "${fn} 执行完成"
            else
                log_cmd_fail "${fn} 执行失败"; exit 1
            fi
        done
        echo ""
        if (( skipped > 0 )); then
            log_info "迁移完成: ${applied} 个新执行, ${skipped} 个已跳过 (共 ${total} 个)"
        else
            log_ok "全部 ${applied} 个迁移执行完成"
        fi
    fi
}

# ── 构建后端 ───────────────────────────────────────────────────────────────────

build_backend() {
    log_step "构建 Rust 后端"

    if [[ ! -f "service/Cargo.toml" ]]; then
        log_error "service/Cargo.toml 不存在"; exit 1
    fi

    # 确保 cargo 在 PATH 中
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    elif [[ -d "$HOME/.cargo/bin" ]] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    if ! command -v cargo &>/dev/null; then
        log_cmd_fail "找不到 cargo，请先安装 Rust"; exit 1
    fi

    log_cmd "编译 nexus-service (cargo build --release)"
    local start; start=$(date +%s)
    if (cd service && cargo build --release --package nexus-service 2>&1 | tail -5); then
        log_cmd_ok "后端编译完成 (耗时 $(( $(date +%s) - start ))s)"
    else
        log_cmd_fail "后端编译失败"; exit 1
    fi
}

# ── 安装前端依赖 ───────────────────────────────────────────────────────────────

install_frontend_deps() {
    log_step "安装前端依赖"

    for app in client admin; do
        if [[ -d "app/${app}" && -f "app/${app}/package.json" ]]; then
            log_cmd "安装 app/${app} 依赖 (npm install)"
            if (cd "app/${app}" && npm install --silent 2>&1 | tail -3); then
                log_cmd_ok "${app} 依赖安装完成"
            else
                log_cmd_fail "${app} 依赖安装失败"; exit 1
            fi
        fi
    done
}

# ── 启动服务 ───────────────────────────────────────────────────────────────────

# ── 主流程 ───────────────────────────────────────────────────────────────────

main() {
    banner "Nexus Setup v0.2"
    cd "$PROJECT_ROOT"
    ensure_project_root

    run_detect
    install_all_deps
    init_config
    setup_database
    build_backend
    install_frontend_deps

    local elapsed; elapsed=$(_nexus_elapsed)
    echo ""
    log_ok "${BOLD}准备完成!${RESET} 总耗时: ${elapsed}"
    echo ""
}

main "$@"
