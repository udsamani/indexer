#!/bin/sh

# Wait for etcd to be ready
sleep 5

# Set the initial configuration
etcdctl put /aionex/indexer/config '{
  "exchanges": [
    {
      "exchange": "kraken",
      "ws_url": "wss://ws.kraken.com/v2",
      "channels": ["ticker"],
      "instruments": ["BTC/USD", "ETH/USD"],
      "heartbeat_millis": 3000
    },
    {
      "exchange": "binance",
      "ws_url": "wss://ws.binance.com",
      "channels": ["ticker"],
      "instruments": ["btcusdt", "ethusdt"],
      "heartbeat_millis": 3000
    },
    {
      "exchange": "coinbase",
      "ws_url": "wss://ws-feed.exchange.coinbase.com",
      "channels": ["ticker", "heartbeat"],
      "instruments": ["BTC-USD", "ETH-USD"],
      "heartbeat_millis": 3000
    }
  ]
}'
