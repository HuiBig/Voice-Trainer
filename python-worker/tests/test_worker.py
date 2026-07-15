import io
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from worker import PROTOCOL_VERSION, handle_request, run


class WorkerProtocolTests(unittest.TestCase):
    def test_ping_round_trip(self):
        response, should_stop = handle_request(
            {
                "protocolVersion": PROTOCOL_VERSION,
                "id": "request-1",
                "method": "ping",
                "params": {"value": "你好"},
            }
        )
        self.assertFalse(should_stop)
        self.assertTrue(response["ok"])
        self.assertEqual(response["result"]["echo"], {"value": "你好"})

    def test_unknown_method_is_structured_error(self):
        response, _ = handle_request(
            {
                "protocolVersion": PROTOCOL_VERSION,
                "id": "request-2",
                "method": "missing",
            }
        )
        self.assertFalse(response["ok"])
        self.assertEqual(response["error"]["code"], "method_not_found")

    def test_stream_emits_ready_error_and_shutdown(self):
        source = io.StringIO(
            "not-json\n"
            + json.dumps(
                {
                    "protocolVersion": PROTOCOL_VERSION,
                    "id": "request-3",
                    "method": "shutdown",
                }
            )
            + "\n"
        )
        output = io.StringIO()
        self.assertEqual(run(source, output), 0)
        messages = [json.loads(line) for line in output.getvalue().splitlines()]
        self.assertEqual(messages[0]["event"], "ready")
        self.assertEqual(messages[1]["error"]["code"], "invalid_json")
        self.assertTrue(messages[2]["result"]["shuttingDown"])


if __name__ == "__main__":
    unittest.main()
