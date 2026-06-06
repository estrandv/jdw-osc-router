#!/usr/bin/env python3
"""Throughput + latency test for jdw-osc-router.

Usage:
    # Start router first:
    ./target/release/jdw-osc-router

    # Then in another terminal:
    python python/throughput_test.py              # 10k msgs, latency + throughput
    python python/throughput_test.py -n 50000      # 50k msgs
    python python/throughput_test.py -s            # sweep multiple counts
"""

import socket
import time
import argparse
import statistics
from pythonosc import udp_client
from pythonosc.osc_packet import OscPacket

HOST = "127.0.0.1"
ROUTER_PORT = 13339


def recv_batch(sock, count_hint=0, timeout=3.0):
    """Receive datagrams, parse OSC, return list of (index, recv_time_ns).

    Drains until 200ms of silence.
    """
    results = []
    idle_start = 0.0
    deadline = time.monotonic() + timeout
    sock.settimeout(0.01)

    while time.monotonic() < deadline:
        try:
            data = sock.recv(65535)
            recv_time = time.monotonic_ns()
            idle_start = 0.0
        except socket.timeout:
            if not results:
                continue
            if idle_start == 0.0:
                idle_start = time.monotonic()
            elif time.monotonic() - idle_start > 0.2:
                break
            continue

        try:
            packet = OscPacket(data)
            for msg in packet.messages:
                params = msg.message.params
                if params and isinstance(params[0], int):
                    results.append((params[0], recv_time))
        except Exception:
            pass

    return results


def run_test(num_messages, subscriber_port):
    test_addr = f"/tput_{subscriber_port}"

    sub_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sub_sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4 * 1024 * 1024)
    sub_sock.bind((HOST, subscriber_port))

    client = udp_client.SimpleUDPClient(HOST, ROUTER_PORT)

    client.send_message("/subscribe", [test_addr, HOST, subscriber_port])
    time.sleep(0.05)

    for _ in range(20):
        client.send_message(test_addr, [0, 0])
    time.sleep(0.1)
    recv_batch(sub_sock, timeout=0.5)

    send_times = [0] * num_messages

    send_start = time.monotonic_ns()
    for i in range(num_messages):
        send_times[i] = time.monotonic_ns()
        client.send_message(test_addr, [i, 0])
    send_end = time.monotonic_ns()

    received = recv_batch(sub_sock, timeout=3.0)

    client.send_message("/unsubscribe", [test_addr, HOST, subscriber_port])
    sub_sock.close()

    send_elapsed = (send_end - send_start) / 1e9
    send_rate = num_messages / send_elapsed if send_elapsed else 0

    latencies = []
    seen = set()
    for idx, recv_time in received:
        if idx not in seen and 0 <= idx < num_messages:
            seen.add(idx)
            lat_ms = (recv_time - send_times[idx]) / 1_000_000
            latencies.append(lat_ms)

    received_count = len(seen)
    loss = num_messages - received_count
    total_elapsed = send_elapsed
    if received:
        last_recv = max(rt for _, rt in received)
        first_send = min(s for s in send_times if s)
        total_elapsed = (last_recv - first_send) / 1e9

    result = {
        "n": num_messages,
        "send_time": send_elapsed,
        "send_rate": send_rate,
        "total_time": total_elapsed,
        "recv_rate": received_count / total_elapsed if total_elapsed else 0,
        "received": received_count,
        "lost": loss,
    }

    if latencies:
        result["lat_avg"] = statistics.mean(latencies)
        result["lat_min"] = min(latencies)
        result["lat_max"] = max(latencies)
        result["lat_p50"] = statistics.median(latencies)
        latencies.sort()
        result["lat_p95"] = latencies[int(len(latencies) * 0.95)]
        result["lat_p99"] = latencies[int(len(latencies) * 0.99)]
    else:
        result["lat_avg"] = result["lat_min"] = result["lat_max"] = 0
        result["lat_p50"] = result["lat_p95"] = result["lat_p99"] = 0

    return result


def print_result(r, label=""):
    tag = f" [{label}]" if label else ""
    pct = 100 * r["received"] / r["n"] if r["n"] else 0
    status = "" if r["lost"] == 0 else f" LOST {r['lost']}"
    print(f"{tag} {r['n']:>6} msgs | "
          f"send {r['send_rate']:>7.0f}/s | "
          f"route {r['recv_rate']:>7.0f}/s | "
          f"rcv {r['received']}/{r['n']} ({pct:5.1f}%){status} | "
          f"lat {r['lat_avg']:6.2f}ms avg "
          f"[p50={r['lat_p50']:5.2f} "
          f"p95={r['lat_p95']:5.2f} "
          f"p99={r['lat_p99']:5.2f} "
          f"max={r['lat_max']:5.2f}]")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="jdw-osc-router throughput + latency test")
    parser.add_argument("-n", type=int, default=10000, help="Messages (default: 10000)")
    parser.add_argument("-p", type=int, default=15455, help="Subscriber port (default: 15455)")
    parser.add_argument("-s", "--sweep", action="store_true",
                        help="Sweep multiple message counts")
    parser.add_argument("--sweep-counts", type=str, default="1000,5000,10000,25000,50000",
                        help="Comma-separated counts for sweep")
    args = parser.parse_args()

    if args.sweep:
        counts = [int(c.strip()) for c in args.sweep_counts.split(",")]
        port = args.p
        print("─" * 120)
        for n in counts:
            r = run_test(n, port)
            print_result(r)
            port += 1
            time.sleep(0.1)
        print("─" * 120)
    else:
        print()
        r = run_test(args.n, args.p)
        print_result(r)
