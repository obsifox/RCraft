#!/usr/bin/env python3
import os
import sys
import json
import socket
import struct
import subprocess
import threading
import time
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

BASE_DIR = "/home/user/rMinecraft"
DOT_MC = os.path.join(BASE_DIR, ".minecraft")
STATIC_DIR = os.path.join(BASE_DIR, "launcher_gui/static")

client_process = None
server_process = None
log_buffer = []
log_lock = threading.Lock()

def append_log(prefix, line):
    with log_lock:
        timestamp = time.strftime("%H:%M:%S")
        entry = f"[{timestamp}] [{prefix}] {line.strip()}"
        log_buffer.append(entry)
        if len(log_buffer) > 500:
            log_buffer.pop(0)

def ping_server(host="127.0.0.1", port=25565, timeout=1.0):
    """Protocol 340 Status Ping"""
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(timeout)
        start = time.time()
        s.connect((host, port))

        # Handshake packet (protocol 340, state 1)
        host_bytes = host.encode('utf-8')
        packet = bytearray()
        packet.append(0x00) # packet ID 0
        # VarInt 340
        packet.extend([0xD4, 0x02])
        # VarInt host len + host
        packet.append(len(host_bytes))
        packet.extend(host_bytes)
        # Port u16
        packet.extend(struct.pack('>H', port))
        # VarInt 1 (Status)
        packet.append(0x01)

        # Length prefix
        full_packet = bytearray([len(packet)]) + packet
        s.sendall(full_packet)

        # Status Request (length 1, id 0)
        s.sendall(bytes([0x01, 0x00]))

        # Read response length
        resp_len = s.recv(1)
        if not resp_len:
            s.close()
            return None
        
        # Read packet
        data = bytearray()
        while len(data) < 2048:
            chunk = s.recv(1024)
            if not chunk:
                break
            data.extend(chunk)

        s.close()
        latency_ms = int((time.time() - start) * 1000)

        # Look for JSON payload inside data
        json_start = data.find(b'{')
        if json_start != -1:
            json_str = data[json_start:].decode('utf-8', errors='ignore')
            json_end = json_str.rfind('}')
            if json_end != -1:
                parsed = json.loads(json_str[:json_end+1])
                parsed['latency_ms'] = latency_ms
                return parsed
        return {"online": True, "latency_ms": latency_ms, "version": {"name": "1.12.2-rust"}}
    except Exception as e:
        return None

class LauncherHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=STATIC_DIR, **kwargs)

    def end_headers(self):
        # Enable CORS and frame embedding for preview
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.send_header('Cache-Control', 'no-cache, no-store, must-revalidate')
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(200)
        self.end_headers()

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path

        if path == "/api/status":
            self.send_json(self.get_status())
        elif path == "/api/accounts":
            self.send_json(self.get_accounts())
        elif path == "/api/collectibles":
            self.send_json(self.get_collectibles())
        elif path == "/api/skins":
            self.send_json(self.get_skins())
        elif path == "/api/logs":
            with log_lock:
                self.send_json({"logs": list(log_buffer)})
        elif path.startswith("/skins/"):
            # Serve local skin
            fname = path[7:]
            fpath = os.path.join(DOT_MC, "skins", fname)
            self.serve_file(fpath, "image/png")
        elif path.startswith("/capes/"):
            fname = path[7:]
            fpath = os.path.join(DOT_MC, "capes", fname)
            self.serve_file(fpath, "image/png")
        elif path == "/" or path == "/index.html":
            index_path = os.path.join(STATIC_DIR, "index.html")
            self.serve_file(index_path, "text/html")
        else:
            super().do_GET()

    def do_POST(self):
        parsed = urlparse(self.path)
        path = parsed.path
        length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(length).decode('utf-8') if length > 0 else "{}"
        try:
            data = json.loads(body)
        except Exception:
            data = {}

        if path == "/api/accounts":
            self.send_json(self.add_account(data))
        elif path == "/api/launch":
            self.send_json(self.launch_client(data))
        elif path == "/api/stop-client":
            self.send_json(self.stop_client())
        elif path == "/api/server/start":
            self.send_json(self.start_server())
        elif path == "/api/server/stop":
            self.send_json(self.stop_server())
        else:
            self.send_error(404, "Endpoint not found")

    def send_json(self, data, code=200):
        body = json.dumps(data).encode('utf-8')
        self.send_response(code)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def serve_file(self, fpath, mime_type):
        if os.path.exists(fpath):
            with open(fpath, 'rb') as f:
                content = f.read()
            self.send_response(200)
            self.send_header('Content-Type', mime_type)
            self.send_header('Content-Length', str(len(content)))
            self.end_headers()
            self.wfile.write(content)
        else:
            self.send_error(404, "File not found")

    def get_status(self):
        global client_process, server_process
        server_running = server_process is not None and server_process.poll() is None
        client_running = client_process is not None and client_process.poll() is None
        server_ping = ping_server("127.0.0.1", 25565) if server_running else None

        return {
            "client_running": client_running,
            "server_running": server_running,
            "server_ping": server_ping,
            "version": "1.12.2-rust",
            "protocol": 340,
            "game_dir": DOT_MC,
            "assets_pack": os.path.exists(os.path.join(DOT_MC, "resourcepacks/modern_textures_1.12.2.zip"))
        }

    def get_accounts(self):
        profiles_file = os.path.join(DOT_MC, "launcher_profiles.json")
        if os.path.exists(profiles_file):
            with open(profiles_file, 'r', encoding='utf-8') as f:
                try:
                    data = json.load(f)
                    return {
                        "accounts": data.get("auth_database", data.get("authenticationDatabase", {})),
                        "profiles": data.get("profiles", {}),
                        "selected_user": data.get("selected_user", data.get("selectedUser", None))
                    }
                except Exception as e:
                    return {"accounts": {}, "profiles": {}, "error": str(e)}
        return {"accounts": {}, "profiles": {}}

    def get_skins(self):
        skins_dir = os.path.join(DOT_MC, "skins")
        capes_dir = os.path.join(DOT_MC, "capes")
        skins = [f for f in os.listdir(skins_dir) if f.endswith('.png')] if os.path.exists(skins_dir) else []
        capes = [f for f in os.listdir(capes_dir) if f.endswith('.png')] if os.path.exists(capes_dir) else []
        return {"skins": skins, "capes": capes}

    def get_collectibles(self):
        return {
            "wallpapers": [
                {
                    "id": "wilderness_bound",
                    "title": "Wilderness Bound",
                    "category": "Key Art / Drop Wallpaper",
                    "image": "/images/wilderness_bound.png",
                    "description": "Lost trails ahead! Alex exploring the amber-hued forest with compass and map."
                },
                {
                    "id": "tiny_takeover",
                    "title": "Tiny Takeover",
                    "category": "Spring Drop Wallpaper",
                    "image": "/images/tiny_takeover.png",
                    "description": "Steve and the iron golem having a peaceful picnic in a cherry grove with tiny animals."
                },
                {
                    "id": "mounts_of_mayhem",
                    "title": "Mounts of Mayhem",
                    "category": "Holiday Drop Wallpaper",
                    "image": "/images/mounts_of_mayhem.png",
                    "description": "Jousting matches in canyons with spears, tridents, and mounted riders."
                },
                {
                    "id": "garden_awakens",
                    "title": "The Garden Awakens",
                    "category": "Pale Garden Drop",
                    "image": "/images/garden_awakens.jpg",
                    "description": "The enigmatic Creaking mob lurking within the mysterious Pale Garden forest."
                },
                {
                    "id": "tricky_trials",
                    "title": "Tricky Trials",
                    "category": "Major Update Wallpaper",
                    "image": "/images/tricky_trials.jpg",
                    "description": "Adventurers battling through copper trial chambers with maces and breeze rods."
                },
                {
                    "id": "bundles_of_bravery",
                    "title": "Bundles of Bravery",
                    "category": "Fall Drop Wallpaper",
                    "image": "/images/bundles_of_bravery.png",
                    "description": "Carrying precious bundles through nighttime wilderness away from creepers."
                },
                {
                    "id": "vanilla_bundle",
                    "title": "Vanilla Bundle Key Art",
                    "category": "Minecraft Classic",
                    "image": "/images/vanilla_bundle.png",
                    "description": "The timeless Minecraft world with Steve, Alex, and the iconic voxel landscape."
                }
            ]
        }

    def add_account(self, data):
        username = data.get("username", "RustDev")
        skin = data.get("skin")
        cape = data.get("cape")
        # Run launcher CLI add-account
        cmd = [
            "/home/user/rMinecraft/target/debug/rminecraft-launcher",
            "--work-dir", DOT_MC,
            "add-account",
            "--username", username
        ]
        if skin:
            cmd.extend(["--skin", os.path.join(DOT_MC, "skins", skin)])
        if cape:
            cmd.extend(["--cape", os.path.join(DOT_MC, "capes", cape)])
        
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, check=True)
            append_log("Launcher", f"Added local account '{username}'")
            return {"success": True, "message": f"Account '{username}' added successfully"}
        except subprocess.CalledProcessError as e:
            return {"success": False, "error": e.stderr}

    def launch_client(self, data):
        global client_process
        if client_process is not None and client_process.poll() is None:
            return {"success": False, "message": "Client is already running"}

        username = data.get("username", "RustDev1")
        server = data.get("server", "127.0.0.1:25565")
        skin = data.get("skin")
        cape = data.get("cape")

        client_bin = os.path.join(BASE_DIR, "target/debug/rminecraft-client")
        cmd = [
            client_bin,
            "--username", username,
            "--server", server,
            "--gameDir", DOT_MC,
            "--assetsDir", os.path.join(DOT_MC, "assets")
        ]
        if skin:
            cmd.extend(["--skin", os.path.join(DOT_MC, "skins", skin)])
        if cape:
            cmd.extend(["--cape", os.path.join(DOT_MC, "capes", cape)])

        append_log("Launcher", f"Starting Native Client: {' '.join(cmd)}")

        def run_proc():
            global client_process
            client_process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1
            )
            for line in client_process.stdout:
                append_log("Client", line)
            client_process.wait()
            append_log("Launcher", f"Client exited with code {client_process.returncode}")

        threading.Thread(target=run_proc, daemon=True).start()
        return {"success": True, "message": f"Launching 1.12.2 client for {username}..."}

    def stop_client(self):
        global client_process
        if client_process is not None and client_process.poll() is None:
            client_process.terminate()
            append_log("Launcher", "Client stopped by user")
            return {"success": True, "message": "Client stopped"}
        return {"success": False, "message": "Client is not running"}

    def start_server(self):
        global server_process
        if server_process is not None and server_process.poll() is None:
            return {"success": False, "message": "Server is already running"}

        server_bin = os.path.join(BASE_DIR, "target/debug/rminecraft-server")
        append_log("Server", "Starting internal 1.12.2 test server...")

        def run_srv():
            global server_process
            server_process = subprocess.Popen(
                [server_bin],
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1
            )
            for line in server_process.stdout:
                append_log("Server", line)
            server_process.wait()
            append_log("Server", f"Server stopped with code {server_process.returncode}")

        threading.Thread(target=run_srv, daemon=True).start()
        return {"success": True, "message": "Server starting on port 25565"}

    def stop_server(self):
        global server_process
        if server_process is not None and server_process.poll() is None:
            server_process.terminate()
            append_log("Server", "Server stopped by user")
            return {"success": True, "message": "Server stopped"}
        return {"success": False, "message": "Server is not running"}

if __name__ == "__main__":
    port = 3000
    # Auto-start server in background for instant play
    server_bin = os.path.join(BASE_DIR, "target/debug/rminecraft-server")
    if os.path.exists(server_bin):
        def auto_srv():
            global server_process
            server_process = subprocess.Popen([server_bin], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            for line in server_process.stdout:
                append_log("Server", line)
        threading.Thread(target=auto_srv, daemon=True).start()

    httpd = HTTPServer(('0.0.0.0', port), LauncherHandler)
    print(f"[Launcher GUI] Running on http://0.0.0.0:{port}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
