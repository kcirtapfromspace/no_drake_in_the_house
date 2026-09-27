"""Exercise nginx cold-start behavior against local HTTP stubs (requires Docker)."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.error import HTTPError
from urllib.request import Request, urlopen
from uuid import uuid4


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--image', required=True, help='An NDITH frontend runtime image')
    parser.add_argument('--use-image-files', action='store_true', help='Check packaged files instead of mounting the checkout')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    wakes = []
    upstream_status = [503]

    class Stub(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            self.respond()

        def do_POST(self):
            self.respond()

        def respond(self):
            body = self.rfile.read(int(self.headers.get('Content-Length', 0)))
            if self.path == '/wake':
                wakes.append({'body': json.loads(body), 'headers': dict(self.headers)})
                status = 200
            else:
                status = upstream_status[0]
            self.send_response(status)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"status":"stub"}')

    server = ThreadingHTTPServer(('0.0.0.0', 0), Stub)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    name = 'ndith-wake-test-' + uuid4().hex[:8]

    def docker(*command):
        return subprocess.run(['docker', *command], check=True, capture_output=True,
                              text=True, timeout=60).stdout.strip()

    try:
        host = 'host.docker.internal' if sys.platform == 'darwin' else docker(
            'network', 'inspect', 'bridge', '--format', '{{(index .IPAM.Config 0).Gateway}}')
        endpoint = f'http://{host}:{server.server_port}'
        command = ['run', '-d', '--name', name, '-p', '127.0.0.1::5000',
                   '-e', 'PORT=5000', '-e', 'GIT_SHA=wake-test',
                   '-e', 'BACKEND_UPSTREAM_URL=' + endpoint,
                   '-e', 'GRAPH_UPSTREAM_URL=http://ndith-nonexistent.invalid:3000',
                   '-e', 'WAKE_URL=' + endpoint + '/wake']
        for source, target in [
            ('frontend/nginx-render.conf.template', '/etc/nginx/templates/default.conf.template'),
            ('frontend/render-entrypoint.sh', '/usr/local/bin/render-entrypoint.sh'),
            ('frontend/warming.html', '/usr/share/nginx/warming.html'),
        ]:
            if not args.use_image_files:
                command.extend(['-v', f'{root / source}:{target}:ro'])
        docker(*command, '--entrypoint', 'sh', args.image, '/usr/local/bin/render-entrypoint.sh')
        port = docker('port', name, '5000/tcp').rsplit(':', 1)[1]

        def request(path, data=None):
            req = Request(f'http://127.0.0.1:{port}' + path, data=data,
                          headers={'Authorization': 'Bearer test-only',
                                   'Cookie': 'session=test-only',
                                   'X-Service-Key': 'test-only',
                                   'Content-Type': 'application/json'})
            try:
                response = urlopen(req, timeout=10)
            except HTTPError as error:
                response = error
            return response.status, response.headers, response.read()

        for _ in range(50):
            try:
                if request('/render-health')[0] == 200:
                    break
            except OSError:
                pass
            time.sleep(0.1)
        else:
            raise AssertionError('nginx did not become ready: ' + docker('logs', name))
        docker('exec', name, 'nginx', '-t')

        cases = [('/api/health', 'Backend', None),
                 ('/api/v1/news/research/trigger?private=query', 'News', b'{"private":"source"}'),
                 ('/api/v1/analytics/status', 'Analytics', None),
                 ('/api/v1/graph/status', 'Graph', None)]
        for path, service, body in cases:
            status, headers, html = request(path, body)
            assert status == 503, (path, status)
            assert headers.get('Retry-After') == '10'
            assert b'Warming up' in html and b'<script>' not in html
            for _ in range(30):
                if any(w['body'] == {'service': service} for w in wakes):
                    break
                time.sleep(0.1)
            else:
                raise AssertionError('Missing wake request for ' + service)
        for wake in wakes:
            names = {key.lower() for key in wake['headers']}
            assert not names.intersection({'authorization', 'cookie', 'x-service-key'})
            assert set(wake['body']) == {'service'}
        print('PASS: cold GET/POST requests wake the correct service without forwarding credentials/body')

        count = len(wakes)
        for _ in range(3):
            assert request('/api/health')[0] == 503
        time.sleep(0.2)
        assert len(wakes) == count, 'Repeated probes were not coalesced'
        print('PASS: repeated probes reuse the wake cache')

        for path in ['/__ndith/wake', '/__ndith/warming.html']:
            assert request(path)[0] == 404, path
        for status in [200, 401, 403]:
            upstream_status[0] = status
            assert request('/api/health')[0] == status
        time.sleep(0.2)
        assert len(wakes) == count
        print('PASS: internal routes are private; healthy/auth responses pass through without waking')

        docker('rm', '-f', name)
        command[command.index('WAKE_URL=' + endpoint + '/wake')] = 'WAKE_URL='
        docker(*command, '--entrypoint', 'sh', args.image, '/usr/local/bin/render-entrypoint.sh')
        port = docker('port', name, '5000/tcp').rsplit(':', 1)[1]
        for _ in range(50):
            try:
                if request('/render-health')[0] == 200:
                    break
            except OSError:
                pass
            time.sleep(0.1)
        upstream_status[0] = 503
        assert request('/api/health')[0] == 503
        time.sleep(0.2)
        assert len(wakes) == count
        print('PASS: deployments without a wake URL retain the 503 response')
    finally:
        subprocess.run(['docker', 'rm', '-f', name], capture_output=True, text=True)
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    main()
