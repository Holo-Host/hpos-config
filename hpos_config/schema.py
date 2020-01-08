__author__                      = "Perry Kundert"
__email__                       = "perry.kundert@holo.host"
__copyright__                   = "Copyright (c) 2020 Holo Limited, Gibralter"
__license__                     = "Apache License, Version 2.0"

import json

CONFIG_FILE = 'hpos-config.json'

def is_email(string):
    return isinstance(string, str) and '@' in string

CONFIG_SCHEMA = {
    'v1': {
        'seed': str,
        'settings': {
            'admin': {
                'email': is_email,
                'public_key': str
            }
        }
    }
}


def check_schema(schema, data, path=''):
    """Validate a schema over the supplied data (assumes JSON, if type(str)); dict/list/value validated
    against dict/list of type, predicate or specific value.

    Raises an Exception on schema failure, with the path indicating where the failure occurred.

    """
    if isinstance(schema, dict) and isinstance(data, dict):
        # schema is a dict of types or other dicts
        for k in schema:
            subpath = f"{path}.{k}"
            assert k in data, f"Missing {subpath} with schema {schema[k]!r}"
            check_schema(schema[k], data[k], path=subpath)
    elif isinstance(schema, list) and isinstance(data, list):
        # schema is list in the form [type or dict].  If schema empty, any list will do
        for i,c in enumerate(data):
            subpath = f"{path}[{i}]"
            if len(schema) == 1: # uniform list
                check_schema(schema[0], c, path=subpath)
            elif i < len(schema): # list of various types; at least schema length required
                check_schema(schema[i], c, path=subpath)
        assert len(schema) <= 1 or len(data) >= len(schema), \
            f"Missing {path}[{len(data)}:{len(schema)}] with schema {', '.join( f'{s!r}' for s in schema[len(data):len(schema)] )}"
    elif isinstance(schema, type):
        # schema is the type of data
        assert isinstance(data, schema), \
            f"Expected {path} of type {schema.__name__}"
    elif hasattr(schema, '__call__'):
        # schema is the predicate for the data
        assert schema(data), \
            f"Expected {path} to satisfy predicate {schema.__name__}"
    else:
        # schema is neither a dict, nor list, not type.  Assume its a value.
        assert schema == data, \
            f"Expected {path} == {schema!r}"


def check_schema_json(schema, data_json, path=''):
    try:
        data = json.loads(data_json)
    except Exception as exc:
        raise RuntimeError(f"Failed to decode JSON{' for ' if path else ''}{path}: {exc}")
    check_schema(schema, data, path)


def check_config(config):
    check_schema(CONFIG_SCHEMA, config, path=f"{CONFIG_FILE}: ")


def check_config_json(config_json):
    check_schema_json(CONFIG_SCHEMA, config_json, path=f"{CONFIG_FILE}: ")
    
