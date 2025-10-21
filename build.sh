#!/bin/bash
set -e  # 遇到错误立即退出

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# 显示帮助信息
show_help() {
    echo "用法: $0 [选项]"
    echo "选项:"
    echo "  -h, --help     显示帮助信息"
    echo "  -v, --version  指定版本号 (默认: v0.3)"
    echo "  -r, --registry 指定镜像仓库 (默认: swr.cn-north-4.myhuaweicloud.com/cloud-blh)"
    echo "  -n, --name     指定镜像名称 (默认: market-saas-backend)"
    echo "  --no-push      只构建不推送"
    echo "  --no-remove    构建后不删除本地镜像"
    echo "示例:"
    echo "  $0 -v v1.0.0"
    echo "  $0 --version v1.0.0 --no-push"
}

# 默认值
VERSION="v0.3"
REGISTRY="swr.cn-north-4.myhuaweicloud.com/cloud-blh"
IMAGE_NAME="market-saas-backend"
DO_PUSH=true
DO_REMOVE=true

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -r|--registry)
            REGISTRY="$2"
            shift 2
            ;;
        -n|--name)
            IMAGE_NAME="$2"
            shift 2
            ;;
        --no-push)
            DO_PUSH=false
            shift
            ;;
        --no-remove)
            DO_REMOVE=false
            shift
            ;;
        *)
            echo -e "${RED}错误: 未知选项 $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# 完整的镜像标签
FULL_IMAGE_TAG="${REGISTRY}/${IMAGE_NAME}:${VERSION}"

echo -e "${YELLOW}=== 构建信息 ===${NC}"
echo -e "镜像标签: ${GREEN}${FULL_IMAGE_TAG}${NC}"
echo -e "推送镜像: ${GREEN}${DO_PUSH}${NC}"
echo -e "删除本地镜像: ${GREEN}${DO_REMOVE}${NC}"
echo

# 检查Docker是否已安装
if ! command -v docker &> /dev/null; then
    echo -e "${RED}错误: Docker未安装或不在PATH中${NC}"
    exit 1
fi

# 检查是否已登录镜像仓库
if $DO_PUSH && ! docker info | grep -q "Username"; then
    echo -e "${YELLOW}警告: 未检测到Docker登录状态，可能需要先登录镜像仓库${NC}"
    echo -e "可以使用以下命令登录: ${GREEN}docker login ${REGISTRY}${NC}"
    read -p "是否继续? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}已取消${NC}"
        exit 1
    fi
fi

# 构建镜像
echo -e "${YELLOW}=== 开始构建镜像 ===${NC}"
if docker build -t "${FULL_IMAGE_TAG}" .; then
    echo -e "${GREEN}镜像构建成功: ${FULL_IMAGE_TAG}${NC}"
else
    echo -e "${RED}镜像构建失败${NC}"
    exit 1
fi

# 推送镜像
if $DO_PUSH; then
    echo -e "${YELLOW}=== 开始推送镜像 ===${NC}"
    if docker push "${FULL_IMAGE_TAG}"; then
        echo -e "${GREEN}镜像推送成功: ${FULL_IMAGE_TAG}${NC}"
    else
        echo -e "${RED}镜像推送失败${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}跳过推送镜像${NC}"
fi

# 删除本地镜像
if $DO_REMOVE; then
    echo -e "${YELLOW}=== 删除本地镜像 ===${NC}"
    if docker rmi "${FULL_IMAGE_TAG}"; then
        echo -e "${GREEN}本地镜像已删除: ${FULL_IMAGE_TAG}${NC}"
    else
        echo -e "${RED}删除本地镜像失败${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}保留本地镜像${NC}"
fi

echo -e "${GREEN}=== 构建流程完成 ===${NC}"

