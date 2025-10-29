"""
Python Arkime Session Module

The Python Arkime Session module has methods for dealing with sessions. The API is very unpythonic and treats the session as a opaque object that needs to be passed around.
"""

from typing import Any, Union, List, Callable

# === Opaque object ===
ArkimeSession = Any

# === Callback type ===
ParserCb = Callable[[ArkimeSession], None]

# === Methods ===

def add_int(session: ArkimeSession, fieldPosOrExp: Union[int, str], value: int) -> None:
    """Add an integer value to a session field."""
    ...

def add_protocol(session: ArkimeSession, protocol: str) -> None:
    """Optimized version of add_string(session, 'protocol', protocol)."""
    ...

def add_string(session: ArkimeSession, fieldPosOrExp: Union[int, str], value: str) -> None:
    """Add a string value to a session field."""
    ...

def add_tag(session: ArkimeSession, tag: str) -> None:
    """Optimized version of add_string(session, 'tags', tag)."""
    ...

def decref(session: ArkimeSession) -> None:
    """Decrement the reference count of a session."""
    ...

def get(session: ArkimeSession, fieldPosOrExp: Union[int, str]) -> Union[Any, List[Any]]:
    """Retrieve the value of a session field."""
    ...

def has_protocol(session: ArkimeSession, protocol: str) -> bool:
    """Check if the session contains the given protocol."""
    ...

def incref(session: ArkimeSession) -> None:
    """Increment the reference count of a session."""
    ...

def register_parser(session: ArkimeSession, parserCb: ParserCb) -> None:
    """Register a parser callback for every packet of the session."""
    ...
