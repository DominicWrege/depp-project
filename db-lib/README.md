### PostgreSQL Configuration Library

Shared code for coniguration the [api]("tree/master/api") and the [Assignment Manager](tree/master/assign-mngr) for establishing the connection to PostgreSQL.

| Name                    | Type   | Default     |
| ----------------------- | ------ | ----------- |
| POSTGRES_USER           | String | johncena    |
| POSTGRES_PASSWORD       | String | secret1     |
| POSTGRES_DB             | String | assignments |
| POSTGRES_PORT           | uint16 | 5432        |
| POSTGRES_HOST           | String | localhost   |
| POSTGRES_MAX_CONNECTION | uint16 | 16          |
