### PostgreSQL Configuration Library

Shared code to configure the REST [API](../api) and the [Assignment Manager](../assign-mngr) for establishing the connection to PostgreSQL using [rust-postgres](https://github.com/sfackler/rust-postgres).

### Config

| Name                    | Type   | Default     |
| ----------------------- | ------ | ----------- |
| POSTGRES_USER           | String | johncena    |
| POSTGRES_PASSWORD       | String | secret1     |
| POSTGRES_DB             | String | assignments |
| POSTGRES_PORT           | uint16 | 5432        |
| POSTGRES_HOST           | String | localhost   |
| POSTGRES_MAX_CONNECTION | uint16 | 16          |
