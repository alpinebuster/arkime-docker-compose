from typing import (
    List, Any, Tuple
)


class Sampler:
    def __init__(
        self, w: int, h: int, k: int, p_1: float, p_2: float
    ) -> None: ...

    def on_pkt_received(
        self,
        flow_key_upstream: str,
        flow_key_downstream: str,
        ts: int  # millisecond
    ) -> None: ...

    def on_pkt_received_raw(
        self,
        src_ip: str,
        dst_ip: str,
        src_port: str,
        dst_port: str,
        ts: int  # millisecond
    ) -> None: ...

    def should_sample(
        self,
        src_ip: str,
        dst_ip: str,
        src_port: str,
        dst_port: str,
    ) -> bool: ...

    def get_efp(
        self,
        src_ip: str,
        dst_ip: str,
        src_port: str,
        dst_port: str,
    ) -> Tuple[int, bool] | None: ...
    def get_efs(self) -> List[str]: ...
    """[[u8, u8, ...], [u8, u8, ...], ...]
    """
    def get_all_nodes(self) -> List[Any]: ...

    def summary(self) -> None: ...
