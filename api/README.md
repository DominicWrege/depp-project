## Description

This is the REST API which uses `JSON`. It communicates via RPC to the Endpoints and acces all assignments from the Postgresql Database.

## Develop

```
# Make sure that the Postgresql instance is running.
cargo run
```

**Endpoint**  
Local:
[http://localhost:6000/api](http://localhost:6000/api) (See Swagger UI for more information.)  
Remote:
[https://auth.inf.fh-dortmund.de:2443/api](https://auth.inf.fh-dortmund.de:2443/api)

# Config

The REST API uses HTTP basic auth passed via environment variable.

### Basic Auth

| Name              | Type   | Default  |
| ----------------- | ------ | -------- |
| DEPP_API_USERNAME | String | user     |
| DEPP_API_PASSWORD | String | wasd4221 |
