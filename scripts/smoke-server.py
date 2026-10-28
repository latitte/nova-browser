"""Pages locales déterministes : le titre est produit par JavaScript."""
from http.server import BaseHTTPRequestHandler, HTTPServer
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        label = 'two' if self.path == '/two' else 'one'
        body = f'<!doctype html><html><title>Chargement</title><body><h1>NOVA {label}</h1><script>document.title="NOVA smoke {label}"</script></body></html>'.encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/html; charset=utf-8')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)
HTTPServer(('127.0.0.1', 18765), Handler).serve_forever()
