#!/usr/bin/env bash

HOST='127.0.0.1'
PORT=6379
AUTH=123456
RCLI="../target/debug/r-cli"

SUCCESS=0
FAIL=0
TOTAL=0

run_test() {
    local cmd="$1"
    ((TOTAL++))

    output=$($RCLI -h $HOST -p $PORT -a $AUTH $cmd)

    echo "== 测试命令: $cmd =="
    echo "输出: $output"

    if [[ "$output" =~ ^\(error\) ]]; then
        echo "❌ FAIL"
        ((FAIL++))
    else
        echo "✅ SUCCESS"
        ((SUCCESS++))
    fi
    echo
}

$RCLI -h $HOST -p $PORT -a $AUTH FLUSHALL

COMMANDS=(
    "AUTH $AUTH"
    "PING"
    "SET foo bar"
    "GET foo"
    "SETNX foo bar2"
    "APPEND foo world"
    "SUBSTR foo 0 2"
    "DEL foo"
    "EXISTS foo"
    "INCR counter"
    "DECR counter"
    "RPUSH mylist a"
    "LPUSH mylist b"
    "LLEN mylist"
    "LINDEX mylist 0"
    "LSET mylist 0 c"
    "LRANGE mylist 0 -1"
    "RPOP mylist"
    "LPOP mylist"
    "BRPOP mylist 1"
    "BLPOP mylist 1"
    "LTRIM mylist 0 1"
    "LREM mylist 0 a"
    "RPOPLPUSH mylist mylist2"
    "SADD myset a"
    "SREM myset a"
    "SMOVE myset myset2 a"
    "SISMEMBER myset a"
    "SCARD myset"
    "SPOP myset"
    "SRANDMEMBER myset"
    "SINTER myset myset2"
    "SINTERSTORE myset3 myset myset2"
    "SUNION myset myset2"
    "SUNIONSTORE myset3 myset myset2"
    "SDIFF myset myset2"
    "SDIFFSTORE myset3 myset myset2"
    "SMEMBERS myset"
    "ZADD myzset 1 one"
    "ZINCRBY myzset 1 one"
    "ZREM myzset one"
    "ZREMRANGEBYSCORE myzset 0 10"
    "ZRANGE myzset 0 -1"
    "ZRANK myzset one"
    "ZREVRANK myzset one"
    "ZRANGEBYSCORE myzset 0 10"
    "ZCOUNT myzset 0 10"
    "ZREVRANGE myzset 0 -1"
    "ZCARD myzset"
    "ZSCORE myzset one"
    "INCRBY counter 10"
    "DECRBY counter 5"
    "GETSET foo bar"
    "RANDOMKEY"
    "SELECT 0"
    "MOVE foo 1"
    "KEYS *"
    "DBSIZE"
    "ECHO hello"
    "SAVE"
    "BGSAVE"
    "BGREWRITEAOF"
    "LASTSAVE"
    "TYPE foo"
    "FLUSHDB"
    "FLUSHALL"
    "SORT mylist"
    "INFO"
    "MGET foo bar"
    "EXPIRE foo 10"
    "EXPIREAT foo 1924992000"
    "TTL foo"
    "SLAVEOF no one"
    "MSET foo1 bar1 foo2 bar2"
    "MSETNX foo1 bar1 foo2 bar2"
    "HSET myhash field1 value1"
    "HGET myhash field1"
    "HDEL myhash field1"
    "HLEN myhash"
    "HKEYS myhash"
    "HVALS myhash"
    "HGETALL myhash"
    "HEXISTS myhash field1"
    "RENAME foo1 bar"
    "RENAMENX foo2 bar"
#    "DEBUG OBJECT foo"
    "MULTI"
#    "EXEC"
#    "DISCARD"
    "SHUTDOWN"
#    "MONITOR"
)

for cmd in "${COMMANDS[@]}"; do
    run_test "$cmd"
done

echo "======================"
echo "Total commands : $TOTAL"
echo "Success        : $SUCCESS"
echo "Failed         : $FAIL"
echo "======================"