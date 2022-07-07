from pythonosc import dispatcher
from pythonosc import osc_server
from pythonosc import udp_client

def print_test(unused_addr, *args):
    print("Test received with args", args)

dispatcher = dispatcher.Dispatcher()

dispatcher.map("/test", print_test)

server = osc_server.ThreadingOSCUDPServer(
    ("127.0.0.1", 13331), dispatcher)
print("Serving on {}".format(server.server_address))

client = udp_client.SimpleUDPClient("127.0.0.1", 13339)
# TODO: Note how python cannot supply a "host port" for the udp client.
client.send_message("/subscribe", ["/test", "13331"])

server.serve_forever()