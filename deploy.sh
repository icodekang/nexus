#!/usr/bin/env bash
# =============================================================================
# Nexus - 一键部署（准备环境 → 启动服务）
# 用法: ./deploy.sh
# =============================================================================
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Fix permissions lost by Windows git (chmod not supported on NTFS)
chmod +x "$SCRIPT_DIR/deploy.sh" "$SCRIPT_DIR/scripts/setup.sh" "$SCRIPT_DIR/scripts/start.sh" "$SCRIPT_DIR/scripts/stop.sh" 2>/dev/null || true

# 清理之前残留的 start.sh 进程
_start_pids=$(ps aux | grep -E "scripts/start\.sh" | grep -v grep | awk '{print $2}' 2>/dev/null) || true
[ -n "$_start_pids" ] && echo "$_start_pids" | xargs kill -9 2>/dev/null || true

# 1. 环境准备
"${SCRIPT_DIR}/scripts/setup.sh" "$@"

# 2. 启动服务
exec "${SCRIPT_DIR}/scripts/start.sh" "$@"
