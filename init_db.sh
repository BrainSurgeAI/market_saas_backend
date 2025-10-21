MYSQL_HOST="127.0.0.1"
MYSQL_PORT="3307"
MYSQL_USER="root"
MYSQL_PASS="Nihaoccj123"

# 执行 SQL 脚本
mysql -h ${MYSQL_HOST} -P ${MYSQL_PORT} -u ${MYSQL_USER} -p"${MYSQL_PASS}" < scripts/database.sql

# 检查执行结果
if [ $? -eq 0 ]; then
    echo "Database initialized successfully!"
else
    echo "Failed to initialize database."
    exit 1
fi