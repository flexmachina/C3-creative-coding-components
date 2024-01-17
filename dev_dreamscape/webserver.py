#!/usr/bin/env python3

import argparse
import http.server
import socketserver

DEFAULT_PORT = 8000


parser = argparse.ArgumentParser("Simple webserver for testing the wasm")
parser.add_argument("-p", "--port", type=int, default=DEFAULT_PORT)
args = parser.parse_args()

Handler = http.server.SimpleHTTPRequestHandler
Handler.extensions_map = {
    '.html': 'text/html',
    '.wasm': 'application/wasm',
    '': 'application/octet-stream',
}

with socketserver.TCPServer(("", args.port), Handler) as httpd:
    # Workaround failure to bind to port immediately after restarting webserver
    httpd.allow_reuse_address = True
    print("serving at port", args.port)
    httpd.serve_forever()
