import sys

import arkime
import arkime_session
import arkime_packet

torch = arkime.get_torch_module()

# Create a new field in the session we will be setting
# REF: `https://arkime.com/taggerformat`
#      `https://arkime.com/settings#custom-fields`
pos = arkime.field_define("dta_rulz", "group:general;kind:lotermfield;db:dta_rulz;friendly:DTA Results;help:DTA results with the power of AI;count:true")
# REF: `https://arkime.com/faq#life-of-a-packet`
#      `https://arkime.com/python`
print("\nDTA Python Module", "VERSION", arkime.VERSION, "CONFIG_PREFIX", arkime.CONFIG_PREFIX, "POS", pos)

print("Torch Version:", torch.__version__)
cuda_available = torch.cuda.is_available()
print("CUDA Available:", cuda_available)


def my_parsers_cb(session, packetBytes, packetLen, direction) -> int:
    # Write code here to parse the bytes and extract information
    print("PARSER:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "len", packetLen, "which", direction)

    # Set a field
    # arkime_session.add_string(session, "dta_rulz", f"True")
    arkime_session.add_string(session, str(pos), f"True")
    arkime_session.add_tag(session, f"packetLen: {packetLen}_{direction}")

    # A parser should return -1 to unregister itself, 0 to continue parsing
    return 0
def my_classify_callback(session, packetBytes, packetLen, direction):
    print("CLASSIFY:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "len", packetLen, "which", direction)

    # Adding a tag
    arkime_session.add_tag(session, f"python_{direction}")

    # Do some kind of check to classify this session by registering the parserCb
    arkime_session.register_parser(session, my_parsers_cb)

def my_pre_save_callback(session, final):
    print("PRE SAVE:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "final", final)
    arkime_session.add_tag(session, f"save FPR: 99%")
def my_save_callback(session, final):
    print("SAVE:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "final", final)

    tls_ja4_pos = arkime.field_get("tls.ja4")
    print("SAVE:", "tls.ja4_pos: ", tls_ja4_pos)

    try:
        print("\n\n\nJA4+ -> ", "tls.ja4: ", arkime_session.get(session, "tls.ja4"), "tls.ja4_r: ", arkime_session.get(session, "tls.ja4_r"), "tcp.ja4t: ", arkime_session.get(session, "tcp.ja4t"), "tcp.ja4l: ", arkime_session.get(session, "tcp.ja4l"), "tcp.ja4ls: ", arkime_session.get(session, "tcp.ja4ls"), "\n\n\n")

        print("!tls.ja4: ", arkime_session.get(session, str(tls_ja4_pos)))
    except Exception as e:
        print("SAVE: Exception getting JA4+ fields:", e)

def my_ethernet_cb(batch, packet, packetBytes, packetLen):
    print("ETHERNET:", "batch", batch, "packet", "packet", "bytes", packetBytes, "len", packetLen, "pktlen", arkime_packet.get(packet, "pktlen"))

    # Remove first 18 bytes of ethernet header and run ethernet callback again
    # bytes = bytes[18:]
    return arkime_packet.run_ethernet_cb(batch, packet, packetBytes, 0, "example_eth")
def my_ip_cb(batch, packet, packetBytes, packetLen):
    print("IP:", "batch", batch, "packet", "packet", "bytes", packetBytes, "len", packetLen, "pktlen", arkime_packet.get(packet, "pktlen"))
    # src = arkime_packet.get(packet, "ip.src")
    # dst = arkime_packet.get(packet, "ip.dst")
    # print("IP_CB:", src, "->", dst, "len", len)

    # arkime_session.add_tag(packet, "python_ip")
    return arkime_packet.run_ip_cb(batch, packet, packetBytes, 0, "example_ip")


### Start ###
# This will match all TCP sessions
arkime.register_tcp_classifier("test", 0, bytes("", "ascii"), my_classify_callback)

arkime.register_pre_save(my_pre_save_callback)
arkime.register_save(my_save_callback)
