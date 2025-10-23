import arkime
import arkime_session
import arkime_packet
import sys

print("\nPython Arkime Module - Example\n")


def my_parsers_cb(session, bytes, len, which):
    # Write code here to parse the bytes and extract information
    print("PARSER:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "len", len, "which", which)

    # then you could set a field
    # arkime_session.add_string(session, pos, "my value")

    # A parser should return -1 to unregister itself, 0 to continue parsing
    return 0
def my_classify_callback(session, bytes, len, which):
    print("CLASSIFY:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "len", len, "which", which)

    # Example of adding a tag
    arkime_session.add_tag(session, "python")

    # Do some kind of check to see if you want to classify this session, if so register
    arkime_session.register_parser(session, my_parsers_cb)


def my_pre_save_callback(session, final):
    print("PRE SAVE:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "final", final)

def my_save_callback(session, final):
    print("SAVE:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "final", final)

def my_ethernet_cb(batch, packet, bytes, len):
    print("ETHERNET:", "batch", batch, "packet", "packet", "bytes", bytes, "len", len, "pktlen", arkime_packet.get(packet, "pktlen"))

    # Remove first 18 bytes of ethernet header and run ethernet callback again
    # bytes = bytes[18:]
    return arkime_packet.run_ethernet_cb(batch, packet, bytes, 0, "example")
# def my_ip_cb(batch, packet, bytes, len):
def my_ip_cb(*args):
    print("my_ip_cb args:", args)
    # src = arkime_packet.get(packet, "ip.src")
    # dst = arkime_packet.get(packet, "ip.dst")
    # print("IP_CB:", src, "->", dst, "len", len)

    # arkime_session.add_tag(packet, "python_ip")
    # return arkime_packet.run_ethernet_cb(batch, packet, bytes, 0, "example")
    return 0


### Start ###
# Register a classifier. This example will match all TCP sessions
# arkime.register_tcp_classifier("test", 0, bytes("", "ascii"), my_classify_callback)

# arkime.register_pre_save(my_pre_save_callback)
# arkime.register_save(my_save_callback)

# EtherType: 0x0800 (IPv4) or 0x86DD (IPv6)
# arkime_packet.set_ethernet_cb(0x0800, my_ethernet_cb)
# 1 = ICMP
# 6 = TCP
# 17 = UDP
# arkime_packet.set_ip_cb(6, my_ip_cb)

# Create a new field in the session we will be setting
pos = arkime.field_define("arkime_rulz", "kind:lotermfield;db:arkime_rulz")

print("VERSION", arkime.VERSION, "CONFIG_PREFIX", arkime.CONFIG_PREFIX, "POS", pos)
