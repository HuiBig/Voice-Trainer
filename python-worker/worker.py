"""Voice Trainer Python worker using a strict JSON Lines protocol."""

from __future__ import annotations

import json
import platform
import sys
from dataclasses import dataclass
from typing import Any, Callable, TextIO

PROTOCOL_VERSION = 1
WORKER_VERSION = "0.1.0"
MAX_LINE_BYTES = 1024 * 1024


@dataclass(frozen=True)
class ProtocolError(Exception):
    code: str
    message: str
    details: Any = None


def _write(message: dict[str, Any], output: TextIO) -> None:
    output.write(json.dumps(message, ensure_ascii=False, separators=(",", ":")) + "\n")
    output.flush()


def _success(request_id: Any, result: Any) -> dict[str, Any]:
    return {
        "protocolVersion": PROTOCOL_VERSION,
        "id": request_id,
        "ok": True,
        "result": result,
    }


def _failure(request_id: Any, error: ProtocolError) -> dict[str, Any]:
    payload: dict[str, Any] = {"code": error.code, "message": error.message}
    if error.details is not None:
        payload["details"] = error.details
    return {
        "protocolVersion": PROTOCOL_VERSION,
        "id": request_id,
        "ok": False,
        "error": payload,
    }


def _ping(params: dict[str, Any]) -> dict[str, Any]:
    return {"pong": True, "echo": params}


def _describe(_: dict[str, Any]) -> dict[str, Any]:
    return {
        "workerVersion": WORKER_VERSION,
        "protocolVersion": PROTOCOL_VERSION,
        "pythonVersion": platform.python_version(),
        "methods": ["describe", "ping", "shutdown"],
    }


HANDLERS: dict[str, Callable[[dict[str, Any]], Any]] = {
    "ping": _ping,
    "describe": _describe,
}


def handle_request(message: Any) -> tuple[dict[str, Any], bool]:
    if not isinstance(message, dict):
        raise ProtocolError("invalid_request", "Request must be a JSON object")

    request_id = message.get("id")
    if not isinstance(request_id, str) or not request_id:
        raise ProtocolError("invalid_request", "Request id must be a non-empty string")
    if message.get("protocolVersion") != PROTOCOL_VERSION:
        return _failure(
            request_id,
            ProtocolError(
                "unsupported_protocol",
                f"Only protocol version {PROTOCOL_VERSION} is supported",
                {"received": message.get("protocolVersion")},
            ),
        ), False

    method = message.get("method")
    if not isinstance(method, str) or not method:
        return _failure(
            request_id, ProtocolError("invalid_request", "Method must be a non-empty string")
        ), False
    params = message.get("params", {})
    if not isinstance(params, dict):
        return _failure(
            request_id, ProtocolError("invalid_params", "Params must be a JSON object")
        ), False

    if method == "shutdown":
        return _success(request_id, {"shuttingDown": True}), True
    handler = HANDLERS.get(method)
    if handler is None:
        return _failure(
            request_id,
            ProtocolError("method_not_found", f"Unknown worker method: {method}"),
        ), False

    try:
        return _success(request_id, handler(params)), False
    except ProtocolError as error:
        return _failure(request_id, error), False
    except Exception as error:  # The protocol must survive a failed job.
        print(f"worker method {method!r} failed: {error}", file=sys.stderr, flush=True)
        return _failure(
            request_id, ProtocolError("internal_error", "Worker method failed")
        ), False


def run(input_stream: TextIO = sys.stdin, output_stream: TextIO = sys.stdout) -> int:
    _write(
        {
            "protocolVersion": PROTOCOL_VERSION,
            "event": "ready",
            "workerVersion": WORKER_VERSION,
            "pid": __import__("os").getpid(),
        },
        output_stream,
    )

    for raw_line in input_stream:
        if len(raw_line.encode("utf-8")) > MAX_LINE_BYTES:
            _write(
                _failure(None, ProtocolError("line_too_large", "Request exceeds 1 MiB")),
                output_stream,
            )
            continue
        try:
            message = json.loads(raw_line)
            response, should_stop = handle_request(message)
        except json.JSONDecodeError as error:
            response = _failure(
                None,
                ProtocolError(
                    "invalid_json",
                    "Request is not valid JSON",
                    {"line": error.lineno, "column": error.colno},
                ),
            )
            should_stop = False
        except ProtocolError as error:
            response = _failure(None, error)
            should_stop = False
        _write(response, output_stream)
        if should_stop:
            return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(run())
