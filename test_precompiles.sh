RPC_URL="http://127.0.0.1:8545"

# 1. 调用 0x100 的 read() 方法
# read() 的 selector = keccak256("read()")[0..4] = 0x57de26a4
curl -s -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"eth_call",
    "params":[
      {
        "to":"0x0000000000000000000000000000000000000100",
        "data":"0x57de26a4"
      },
      "latest"
    ],
    "id":1
  }' | jq

# 2. 调用 0x200 的 setNum(uint64) 方法
# 假设我们要设置 num = 42
# setNum(uint64) selector = keccak256("setNum(uint64)")[0..4] = 0x733954ea
# uint64 参数填充到 32 字节，右对齐
NUM=79
PARAM=$(printf "%064x" $NUM)
curl -s -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"eth_call",
    "params":[
      {
        "to":"0x0000000000000000000000000000000000000200",
        "data":"0x733954ea'"$PARAM"'"
      },
      "latest"
    ],
    "id":2
  }' | jq

# 3. 调用 0x200 的 getNum() 方法
# getNum() selector = keccak256("getNum()")[0..4] = 0x67e0badb
curl -s -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"eth_call",
    "params":[
      {
        "to":"0x0000000000000000000000000000000000000200",
        "data":"0x67e0badb"
      },
      "latest"
    ],
    "id":3
  }' | jq