# Indexer

## Overview

The indexer is a service that indexes the data from the exchanges and makes it available to the rest of the system. It is responsible for the following:

- Receiving data from the exchanges
- Smoothing the data
- Calculating the weighted average
- Distributing the data to an endpoint

## Smoothing

The indexer supports the following smoothing algorithms:

- Exponential Moving Average (EMA)
- Simple Moving Average (SMA)
- PassThru (no smoothing)

## Worker Architecture

![Worker Architecture](./docs/worker-architecture.png)


## How to run the indexer ?

1. Build Indexer Binary

```bash
make build
```

2. Run peripheral services

```bash
make services
```

This spins up the following services:

- etcd
- postgres
- grafana
- prometheus
- jsonrpc-mock-server

3. Load initial app config to etcd

```bash
make etcd-init
```
This loads the initial app config to etcd, which is

```json
{
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
```
Please note that for this to run, you will need to have `etcdctl` installed and configured to connect to the etcd instance
spun up by the `make services` command. It spins it up on port 2379.

4. Run Indexer

```bash
cargo run --bin indexer
```
This will start the indexer and it will be ready to receive data from the exchanges.


## Some information about the configuration

- At the moment the pricer weight is defined per exchange and not per instrument.
- If you remove all instruments from an exchange and do not change the weight for that exchange, the app will exit gracefully
  error that total weight does not add up to 100.
- Removal of one of the exchange entirely does not disconnect the indexer from the exchange. It will continue to run and will not return on error.
- Thus it is advised to not only change channels, smoother config, weights, or instruments. The url and other config once set initially do not have any impact if changed.
- The entire blob of the config needs to be present when updating the config. 
- The config key is `/aionex/indexer/config`


## Static configuration

- Certain configuration is static and does not change and needs be present when the app starts.
- This information is present in `.env/indexer.env`.

## Distribution via endpoints

- The indexer distributes the data via endpoints.
- Currently the indexer is configured to distribute the data to a mock-jsonrpc server, which is spun up by the `make services` command.
- To see that indexer is distributing the data correctly, you can take a look at the logs of the mock-jsonrpc server.
- ``` docker container logs -f docker-jsonrpc-mock-server-1 ```

## Raw Feed into the databse

- The indexer also has a feature to dump the raw feed into the database.
- You can connect to the database using `psql` and take a look at the `tickers` table in the `indexer` database.

``` psql postgresql://postgres:postgres@localhost:5432/indexer```

## Grafana

- The indexer comes with a grafana dashboard.
- You can load up grafana on a browser using the following url:

``` http://localhost:3000/ ```

- The username and password are `admin` and `admin` respectively.

## Prometheus

- The indexer comes with a prometheus instance.
- You can access the prometheus instance on a browser using the following url:

``` http://localhost:9090/ ```
