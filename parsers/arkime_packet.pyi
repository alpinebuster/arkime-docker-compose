"""
Python Arkime Packet Module

The Python Arkime Packet module has methods for dealing with packets before they are associated with sessions. The API is very unpythonic and treats the packet as a opaque object that needs to be passed around.
"""

from typing import Any, Callable

# === Constants for PacketRC ===
PacketRC = int
DO_PROCESS: PacketRC      # Process the packet normally
CORRUPT: PacketRC         # The packet is corrupt
UNKNOWN: PacketRC         # The packet is unknown and can't be processed
DONT_PROCESS: PacketRC    # The packet should not be processed but can be freed
DONT_PROCESS_OR_FREE: PacketRC  # The packet should not be processed and should not be freed

# === Opaque objects ===
ArkimePacketBatch = Any
ArkimePacket = Any

# === Callback type ===
PacketCb = Callable[[ArkimePacketBatch, ArkimePacket, memoryview, int], PacketRC]

# === Methods ===

def get(packet: ArkimePacket, field: str) -> Any:
    """
    Retrieve the value of a packet field.

    Args:
        packet: The packet object from the packet callback.
        field: The string name of the field to retrieve.

    Returns:
    """
    ...
def set(packet: ArkimePacket, field: str, value: int) -> None:
    """
    Set the value of a packet field. Not all fields can be set.

    Args:
        packet: The packet object from the packet callback.
        field: The string name of the field to set.
        value: The integer value to set.
    """
    ...

def set_ethernet_cb(type: int, packetCb: PacketCb) -> None:
    """
    Register an ethertype packet callback that will be called for packets of the given type. Usually this callback with just need to strip some headers and call either run_ip_cb or run_ethernet_cb.

    Args:
        type: The Ethertype to register the callback for.
        packetCb: The callback to call for packets of the given ethertype.
    """
    ...
def set_ip_cb(type: int, ipCb: PacketCb) -> None:
    """
    Register an IP protocol packet callback that will be called for packets of the given protocol.

    Args:
        type: The IP protocol to register the callback for.
        ipCb: The callback to call for packets of the given protocol.
    """
    ...

def run_ethernet_cb(
    batch: ArkimePacketBatch,
    packet: ArkimePacket,
    packetBytes: memoryview,
    packetLen: int,
    description: str
) -> PacketRC:
    """
    Process a packet at the Ethernet layer by running the registered Ethernet callback.

    Args:
        batch: The opaque batch object.
        packet: The opaque packet object.
        packetBytes: The memory view of the packet bytes
        packetLen: The length of the packet in bytes.
        description: A string description of the packet.

    Returns:
    """
    ...
def run_ip_cb(
    batch: ArkimePacketBatch,
    packet: ArkimePacket,
    packetBytes: memoryview,
    packetLen: int,
    description: str
) -> PacketRC:
    """
    Process a packet at the IP layer by running the registered IP callback.

    Args:
        batch: The opaque batch object.
        packet: The opaque packet object.
        packetBytes: A memoryview of the packet bytes.
        packetLen: The length of the packet in bytes.
        description: A string description of the packet.

    Returns:
    """
    ...
