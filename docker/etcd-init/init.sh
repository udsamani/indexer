#!/bin/sh

# Wait for etcd to be ready
sleep 5

# Set the initial configuration
etcdctl put /aionex/indexer/config '{
  "kraken": {
      "exchange_config": {
          "ws_url": "wss://ws.kraken.com/v2",
          "channels": ["ticker"],
          "instruments": ["BTC/USD", "ETH/USD"],
          "heartbeat_millis": 3000
      },
      "smoothing_config": {
          "type": "ema",
          "params": {
              "window": 100,
              "smoothing": 2.0
          }
      },
      "weight": 30.0
  },
  "binance": {
      "exchange_config": {
          "ws_url": "wss://stream.binance.com:9443/ws",
          "channels": ["ticker"],
          "instruments": ["ETHUSDT", "BTCUSDT"],
          "heartbeat_millis": 3000
      },
      "smoothing_config": {
          "type": "sma",
          "params": {
              "window": 100
          }
      },
      "weight": 40.0
  },
  "coinbase": {
      "exchange_config": {
          "ws_url": "wss://ws-feed.exchange.coinbase.com",
          "channels": ["ticker", "heartbeat"],
          "instruments": ["BTC-USD", "ETH-USD"],
          "heartbeat_millis": 3000
      },
      "smoothing_config": {
          "type": "sma",
          "params": {
              "window": 20
          }
      },
      "weight": 30.0
  }
}'


etcdctl put /aionex/indexer/config '{
  "kraken": {
      "exchange_config": {
          "ws_url": "wss://ws.kraken.com/v2",
          "channels": ["ticker"],
          "instruments": ["BTC/USD", "ETH/USD"],
          "heartbeat_millis": 3000
      },
      "smoothing_config": {
          "type": "ema",
          "params": {
              "window": 100,
              "smoothing": 2.0
          }
      },
      "weight": 30.0
  },
  "binance": {
      "exchange_config": {
          "ws_url": "wss://stream.binance.com:9443/ws",
          "channels": ["ticker"],
          "instruments": ["ETHUSDT", "BTCUSDT"],
          "heartbeat_millis": 3000
      },
      "smoothing_config": {
          "type": "sma",
          "params": {
              "window": 100
          }
      },
      "weight": 40.0
  },
  "coinbase": {
      "exchange_config": {
          "ws_url": "wss://ws-feed.exchange.coinbase.com",
          "channels": ["ticker", "heartbeat"],
          "instruments": ["BTC-USD", "ETH-USD"],
          "heartbeat_millis": 3000
      },
      "smoothing_config": {
          "type": "pass_thru"
      },
      "weight": 30.0
  }
}'