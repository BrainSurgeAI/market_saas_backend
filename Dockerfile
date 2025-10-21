# 后端 Dockerfile
FROM rust:1.84.1-alpine3.20 AS builder
WORKDIR /app
# 设置Alpine镜像源以加速依赖安装
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    gcc \
    make \
    libc-dev \
    linux-headers

# 配置Cargo国内镜像源 - 直接创建配置文件
RUN mkdir -p /usr/local/cargo/ && \
    echo '[source.crates-io]' > /usr/local/cargo/config.toml && \
    echo 'replace-with = "ustc"' >> /usr/local/cargo/config.toml && \
    echo '[source.ustc]' >> /usr/local/cargo/config.toml && \
    echo 'registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"' >> /usr/local/cargo/config.toml && \
    echo '[registries.ustc]' >> /usr/local/cargo/config.toml && \
    echo 'index = "https://mirrors.ustc.edu.cn/crates.io-index/"' >> /usr/local/cargo/config.toml && \
    echo '[net]' >> /usr/local/cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> /usr/local/cargo/config.toml

# 先复制Cargo.toml和Cargo.lock以利用Docker缓存
COPY Cargo.toml Cargo.lock ./
# 创建一个虚拟的src/main.rs以便编译依赖
RUN mkdir -p src && echo 'fn main() { println!("Dummy!"); }' > src/main.rs
# 编译依赖
RUN cargo build --release
# 删除虚拟的构建结果
RUN rm -rf src target/release/deps/market_saas_backend*

# 复制实际源代码
COPY . .

ENV SQLX_OFFLINE=true
# 安装sqlx-cli工具
RUN cargo install sqlx-cli --no-default-features --features mysql

# 重新编译，这次只编译应用代码，依赖已经缓存
RUN cargo build --release

FROM alpine:3.20
WORKDIR /app
# 设置Alpine镜像源以加速依赖安装
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# 安装运行时依赖
RUN apk update && apk add --no-cache openssl ca-certificates wget mysql-client bash

# 复制二进制文件
COPY --from=builder /app/target/release/market-saas-backend .
# 复制sqlx-cli工具
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/

# 复制必要的目录
COPY --from=builder /app/migrations/ /app/migrations/
# 创建scripts目录并复制脚本文件
RUN mkdir -p /app/scripts /app/config
COPY scripts/migrate.sh /app/scripts/
COPY scripts/entrypoint.sh /app/scripts/
COPY permissions.yaml /app/config/permissions.yaml

# 创建必要的目录
RUN mkdir -p /app/logs /app/data

# 使用非root用户运行应用（安全最佳实践）
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
RUN chown -R appuser:appgroup /app

# 确保脚本可执行
RUN chmod +x /app/scripts/migrate.sh
RUN chmod +x /app/scripts/entrypoint.sh

USER appuser

# 设置环境变量
ENV RUST_BACKTRACE=1
ENV LOG_DIR=/app/logs
ENV DATA_DIR=/app/data
# 默认禁用迁移，由初始化容器执行
ENV PERFORM_MIGRATION=false

# 添加健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# 使用启动脚本
CMD ["/app/scripts/entrypoint.sh"]