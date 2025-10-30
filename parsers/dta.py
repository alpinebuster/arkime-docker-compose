import sys

import arkime
import arkime_session
import arkime_packet

# Create a new field in the session we will be setting
# REF: `https://arkime.com/taggerformat`
#      `https://arkime.com/settings#custom-fields`
pos = arkime.field_define("dta_rulz", "kind:lotermfield;group:general;db:dta_rulz;count:true;friendly:DTA Results;help:DTA results with the power of AI")
# REF: `https://arkime.com/faq#life-of-a-packet`
#      `https://arkime.com/python`
print("\nDTA Python Module", "VERSION", arkime.VERSION, "CONFIG_PREFIX", arkime.CONFIG_PREFIX, "POS", pos)

def my_parsers_cb(session, packetBytes, packetLen, direction) -> int:
    # Write code here to parse the bytes and extract information
    print("PARSER:", arkime_session.get(session, "ip.src"), ":", arkime_session.get(session, "port.src"), "->", arkime_session.get(session, "ip.dst"), ":", arkime_session.get(session, "port.dst"), "len", packetLen, "which", direction)

    # Set a field
    arkime_session.add_string(session, "dta_rulz", f"True")
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


# /* Ethernet protocol ID's */
# #define	ETHERTYPE_PUP		0x0200          /* Xerox PUP */
# #define ETHERTYPE_SPRITE	0x0500		/* Sprite */
# #define	ETHERTYPE_IP		0x0800		/* IP */
# #define	ETHERTYPE_ARP		0x0806		/* Address resolution */
# #define	ETHERTYPE_REVARP	0x8035		/* Reverse ARP */
# #define ETHERTYPE_AT		0x809B		/* AppleTalk protocol */
# #define ETHERTYPE_AARP		0x80F3		/* AppleTalk ARP */
# #define	ETHERTYPE_VLAN		0x8100		/* IEEE 802.1Q VLAN tagging */
# #define ETHERTYPE_IPX		0x8137		/* IPX */
# #define	ETHERTYPE_IPV6		0x86dd		/* IP protocol version 6 */
# #define ETHERTYPE_LOOPBACK	0x9000		/* used to test interfaces */
#
# #define	ETHER_ADDR_LEN	ETH_ALEN                 /* size of ethernet addr */
# #define	ETHER_TYPE_LEN	2                        /* bytes in type field */
# #define	ETHER_CRC_LEN	4                        /* bytes in CRC field */
# #define	ETHER_HDR_LEN	ETH_HLEN                 /* total octets in header */
# #define	ETHER_MIN_LEN	(ETH_ZLEN + ETHER_CRC_LEN) /* min packet length */
# #define	ETHER_MAX_LEN	(ETH_FRAME_LEN + ETHER_CRC_LEN) /* max packet length */
#
#arkime_packet.set_ethernet_cb(0x0800, my_ethernet_cb)


# /* Standard well-defined IP protocols.  */
# enum
#   {
#     IPPROTO_IP = 0,	   /* Dummy protocol for TCP.  */
# #define IPPROTO_IP		IPPROTO_IP
#     IPPROTO_ICMP = 1,	   /* Internet Control Message Protocol.  */
# #define IPPROTO_ICMP		IPPROTO_ICMP
#     IPPROTO_IGMP = 2,	   /* Internet Group Management Protocol. */
# #define IPPROTO_IGMP		IPPROTO_IGMP
#     IPPROTO_IPIP = 4,	   /* IPIP tunnels (older KA9Q tunnels use 94).  */
# #define IPPROTO_IPIP		IPPROTO_IPIP
#     IPPROTO_TCP = 6,	   /* Transmission Control Protocol.  */
# #define IPPROTO_TCP		IPPROTO_TCP
#     IPPROTO_EGP = 8,	   /* Exterior Gateway Protocol.  */
# #define IPPROTO_EGP		IPPROTO_EGP
#     IPPROTO_PUP = 12,	   /* PUP protocol.  */
# #define IPPROTO_PUP		IPPROTO_PUP
#     IPPROTO_UDP = 17,	   /* User Datagram Protocol.  */
# #define IPPROTO_UDP		IPPROTO_UDP
#     IPPROTO_IDP = 22,	   /* XNS IDP protocol.  */
# #define IPPROTO_IDP		IPPROTO_IDP
#     IPPROTO_TP = 29,	   /* SO Transport Protocol Class 4.  */
# #define IPPROTO_TP		IPPROTO_TP
#     IPPROTO_DCCP = 33,	   /* Datagram Congestion Control Protocol.  */
# #define IPPROTO_DCCP		IPPROTO_DCCP
#     IPPROTO_IPV6 = 41,     /* IPv6 header.  */
# #define IPPROTO_IPV6		IPPROTO_IPV6
#     IPPROTO_RSVP = 46,	   /* Reservation Protocol.  */
# #define IPPROTO_RSVP		IPPROTO_RSVP
#     IPPROTO_GRE = 47,	   /* General Routing Encapsulation.  */
# #define IPPROTO_GRE		IPPROTO_GRE
#     IPPROTO_ESP = 50,      /* encapsulating security payload.  */
# #define IPPROTO_ESP		IPPROTO_ESP
#     IPPROTO_AH = 51,       /* authentication header.  */
# #define IPPROTO_AH		IPPROTO_AH
#     IPPROTO_MTP = 92,	   /* Multicast Transport Protocol.  */
# #define IPPROTO_MTP		IPPROTO_MTP
#     IPPROTO_BEETPH = 94,   /* IP option pseudo header for BEET.  */
# #define IPPROTO_BEETPH		IPPROTO_BEETPH
#     IPPROTO_ENCAP = 98,	   /* Encapsulation Header.  */
# #define IPPROTO_ENCAP		IPPROTO_ENCAP
#     IPPROTO_PIM = 103,	   /* Protocol Independent Multicast.  */
# #define IPPROTO_PIM		IPPROTO_PIM
#     IPPROTO_COMP = 108,	   /* Compression Header Protocol.  */
# #define IPPROTO_COMP		IPPROTO_COMP
#     IPPROTO_SCTP = 132,	   /* Stream Control Transmission Protocol.  */
# #define IPPROTO_SCTP		IPPROTO_SCTP
#     IPPROTO_UDPLITE = 136, /* UDP-Lite protocol.  */
# #define IPPROTO_UDPLITE		IPPROTO_UDPLITE
#     IPPROTO_MPLS = 137,    /* MPLS in IP.  */
# #define IPPROTO_MPLS		IPPROTO_MPLS
#     IPPROTO_ETHERNET = 143, /* Ethernet-within-IPv6 Encapsulation.  */
# #define IPPROTO_ETHERNET	IPPROTO_ETHERNET
#     IPPROTO_RAW = 255,	   /* Raw IP packets.  */
# #define IPPROTO_RAW		IPPROTO_RAW
#     IPPROTO_MPTCP = 262,   /* Multipath TCP connection.  */
# #define IPPROTO_MPTCP		IPPROTO_MPTCP
#     IPPROTO_MAX
#   };
# 
# FIXME
#arkime_packet.set_ip_cb(0x06, my_ip_cb)
