from pythonosc import dispatcher
from pythonosc import osc_server
from pythonosc import udp_client

# Starts an OSC server that immediately subscribes to the /test address in the router.
# sendtest.py demonstrates how to send such messages to test the full routing flow.

def print_test(unused_addr, *args):
    print("/test received with args: ", args)

dispatcher = dispatcher.Dispatcher()

dispatcher.map("/test", print_test)

server = osc_server.ThreadingOSCUDPServer(
    ("127.0.0.1", 13331), dispatcher)
print("Serving on {}".format(server.server_address))

client = udp_client.SimpleUDPClient("127.0.0.1", 13339)

# Subscribe self (note same port as in "server")
client.send_message("/subscribe", ["/test", "127.0.0.1", 13331])

server.serve_forever()