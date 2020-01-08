import json
import pytest
from hpos_config import schema

config_json = """\

{
  "v1": {
    "seed": "YXxZUB9s0eCnJo+lAhlAt2FJKLqT8KP1x2X3qzAZNQs",
    "settings": {
      "admin": {
        "email": "test@example.com",
        "public_key": "uaUz9m8QNPXNvyuTbdNb3MpQ7iCe6UmWhdQ7ChUlpaQ"
      },
      "other": {
        "ints": [1,2],
        "stuff": [133]
      }
    }
  }
}
"""

def test_config():
    schema.check_config_json(config_json)

    config = json.loads(config_json)
    config['v1']['settings']['admin']['email'] = 'boo'
    with pytest.raises(Exception) as exc:
        schema.check__config(config)


def test_schema_lists():
    schema.check_schema(
        {'list': { 'stuff': [int, str, float]}},
        {'list': { 'stuff': [1, 'a', 1.2, 'something'] }}
    )
    with pytest.raises(Exception) as exc:
        schema.check_schema(
            {'list': { 'stuff': [int, str, float]}},
            {'list': { 'stuff': [1, 'a', 'b'] }}
        )
    with pytest.raises(Exception) as exc:
        schema.check_schema(
            {'list': { 'stuff': [int, str, float]}},
            {'list': { 'stuff': [1, 'a'] }}
        )
    schema.check_schema(
        {'list': { 'ints': [int]}},
        {'list': { 'ints': [] }}
    )
    schema.check_schema(
        {'list': { 'ints': [int]}},
        {'list': { 'ints': [1,1] }}
    )
