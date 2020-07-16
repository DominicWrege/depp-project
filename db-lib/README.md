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

## Backup And Restore Database

**_Note_**: Make sure you have the matching postgresql-client installed.
See how to [install postgresql 12 on Ubuntu.](https://computingforgeeks.com/install-postgresql-12-on-ubuntu/)

Backup:

```
pg_dump postgres://USER:"PASSWORD"@127.0.0.1:5432/assignments > db-backup
```

Restore:

```
psql postgres://USER:"PASSWORD"@127.0.0.1:5432/assignments < db-backup
```

Read the PostgrSQL [documentation](https://www.postgresql.org/docs/current/backup-dump.html)..
