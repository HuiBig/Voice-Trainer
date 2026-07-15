# Python Worker protocol

The worker owns Python/RVC execution and communicates exclusively through UTF-8 JSON Lines.
`stdout` is reserved for protocol messages; diagnostics go to `stderr`.

Each request has this shape:

```json
{"protocolVersion":1,"id":"1","method":"ping","params":{}}
```

Each response repeats the request `id` and contains either `ok: true` plus `result`, or
`ok: false` plus a stable `error.code`. The worker emits one `ready` event when it starts.
Lines larger than 1 MiB are rejected. Protocol version changes require a new
`protocolVersion`; adding a method does not.
