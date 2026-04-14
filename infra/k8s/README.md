# Nexus Kubernetes 部署配置

## 目录结构

```
infra/k8s/
├── base/                    # 基础配置（所有环境通用）
│   ├── kustomization.yaml
│   ├── namespace.yaml
│   ├── configmap.yaml       # 非敏感配置
│   ├── secret.yaml          # 密钥模板（勿填真实值）
│   ├── postgres.yaml        # PostgreSQL StatefulSet + Service
│   ├── redis.yaml           # Redis StatefulSet + Service
│   ├── nexus-service.yaml   # Rust API Gateway
│   ├── nexus-client.yaml    # React 前台 (SPA)
│   ├── nexus-admin.yaml     # React 管理后台 (SPA)
│   ├── ingress.yaml         # Ingress (nginx)
│   └── hpa.yaml             # 自动扩缩容
└── overlays/
    ├── prod/                # 生产环境
    │   ├── kustomization.yaml
    │   └── secret-patch.yaml
    └── dev/                 # 开发环境
        └── kustomization.yaml
```

## 前置要求

- Kubernetes 1.28+
- kubectl 已配置集群
- Ingress NGINX Controller 已安装
- Registry 已推送镜像（或使用本地 kind/Docker Desktop）

### 安装 Ingress Controller

```bash
# 使用 helm
helm upgrade --install ingress-nginx ingress-nginx \
  --repo https://kubernetes.github.io/ingress-nginx \
  --namespace ingress-nginx --create-namespace \
  --set controller.publishService.enabled=true

# 或使用官方 yaml
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.9.0/deploy/static/provider/cloud/deploy.yaml
```

### 安装 metrics-server（HPA 需要）

```bash
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
```

## 构建并推送镜像

```bash
# 构建 API 镜像
docker build -t ghcr.io/icodekang/nexus-service:latest ./service/

# 构建前端镜像
docker build -t ghcr.io/icodekang/nexus-client:latest ./app/client/
docker build -t ghcr.io/icodekang/nexus-admin:latest ./app/admin/

# 推送（需要登录 GHCR）
docker push ghcr.io/icodekang/nexus-service:latest
docker push ghcr.io/icodekang/nexus-client:latest
docker push ghcr.io/icodekang/nexus-admin:latest
```

## 部署

### 开发环境

```bash
# 加载本地镜像到 kind
kind load docker-image nexus-service:latest --name kind-nexus

# 部署
kubectl apply -k infra/k8s/overlays/dev/

# 查看状态
kubectl get pods -n nexus-dev

# 端口转发（本地访问）
kubectl port-forward -n nexus-dev svc/dev-nexus-client 3001:80 &
kubectl port-forward -n nexus-dev svc/dev-nexus-service 8080:8080 &
```

### 生产环境

```bash
# 1. 编辑 secret-patch.yaml 填入真实密钥
vim infra/k8s/overlays/prod/secret-patch.yaml
kubectl apply -f infra/k8s/overlays/prod/secret-patch.yaml -n nexus

# 2. 部署
kubectl apply -k infra/k8s/overlays/prod/

# 3. 验证
kubectl get pods -n nexus
kubectl get ingress -n nexus

# 4. 配置域名 DNS 解析到 Ingress IP
#   nexus.example.com      → Ingress IP
#   admin.nexus.example.com → Ingress IP

# 5. 配置 TLS（生产必须）
#    推荐使用 cert-manager 自动签发 Let's Encrypt 证书
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/latest/download/cert-manager.yaml
```

## 访问

| 端点 | 路径 |
|------|------|
| 用户前台 | `https://nexus.example.com/` |
| 管理后台 | `https://admin.nexus.example.com/` 或 `https://nexus.example.com/admin/` |
| API 健康检查 | `https://nexus.example.com/v1/` |
| API 文档 | 参考 `docs/API.md` |

## 扩缩容

```bash
# 手动扩缩容
kubectl scale deployment nexus-service -n nexus --replicas=5

# HPA 自动管理（CPU/memory）
kubectl get hpa -n nexus
```

## 清理

```bash
# 开发环境
kubectl delete -k infra/k8s/overlays/dev/

# 生产环境
kubectl delete -k infra/k8s/overlays/prod/
```

## 生产环境注意事项

1. **密钥管理**：使用 SealedSecrets / ExternalSecrets / Vault 管理密钥，不要 commit 明文密钥
2. **数据库**：生产建议使用云托管数据库（RDS / Cloud SQL），删除 `postgres.yaml`
3. **Redis**：生产建议使用云托管缓存（ElastiCache / Memorystore），删除 `redis.yaml`
4. **镜像签名**：启用 Cosign 验证镜像来源
5. **网络策略**：配置 Kubernetes NetworkPolicy 限制 Pod 间访问
6. **Pod 安全**：启用 PSP 或 Pod Security Standards
7. **备份**：配置 PostgreSQL 定期备份策略
