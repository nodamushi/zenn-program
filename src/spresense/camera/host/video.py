import serial
import threading
import time
import os
from datetime import datetime
import cv2
import numpy as np
import struct
from enum import Enum
import queue
from flask import Flask, render_template_string, jsonify, request, Response

class RecordingState(Enum):
    IDLE = 0
    RECORDING = 1


class ResponseCode(Enum):
    START_SUCCESS = "rsp:VST."
    START_FAILURE = "rsp:VST!"
    ALREADY_RECORDING = "rsp:VST#"
    STOP_SUCCESS = "rsp:VSP."
    STOP_FAILURE = "rsp:VSP!"
    NOT_RECORDING = "rsp:VSP#"
    SNAPSHOT_SUCCESS = "rsp:SST."
    SNAPSHOT_FAILURE = "rsp:SST!"


class WhiteBalanceMode(Enum):
    AUTO = 'B'
    INCANDESCENT = 'C'
    FLUORESCENT = 'D'
    DAYLIGHT = 'G'
    CLOUDY = 'I'
    SHADE = 'J'


END_MARKER_VALUE = 0xFA01FB00
SNAPSHOT_FRAME_NUMBER = 0xFFFFFFFF


class SerialCommunication:
    def __init__(self, port, baudrate=9800, timeout=1):
        self.port = port
        self.baudrate = baudrate
        self.timeout = timeout
        self.serial_port = None
        self.is_connected = False
        self.receive_callback = None
        self.stop_thread = False
        self.receive_thread = None
        self.debug_callback = None
        self.buffer_size = 0

    def set_debug_callback(self, callback):
        self.debug_callback = callback

    def _debug(self, message):
        if self.debug_callback:
            self.debug_callback(f"[COMM] {message}")

    def connect(self):
        try:
            self._debug(f"Connecting to {self.port} at baudrate {self.baudrate}...")
            self.serial_port = serial.Serial(
                port=self.port,
                baudrate=self.baudrate,
                timeout=self.timeout
            )
            self.is_connected = True
            self.stop_thread = False
            self._debug("Serial port opened successfully")
            self.receive_thread = threading.Thread(target=self._receive_data)
            self.receive_thread.daemon = True
            self.receive_thread.start()
            self._debug("Receive thread started")
            return True
        except Exception as e:
            self._debug(f"Connection error: {e}")
            return False

    def disconnect(self):
        self._debug("Disconnecting...")
        self.stop_thread = True
        if self.receive_thread:
            self.receive_thread.join(timeout=1.0)
        if self.serial_port and self.serial_port.is_open:
            self.serial_port.close()
        self.is_connected = False
        self._debug("Disconnected")

    def send_data(self, data):
        if self.is_connected and self.serial_port:
            try:
                self._debug(f"Sending: {data}")
                self.serial_port.write(data.encode())
                return True
            except Exception as e:
                self._debug(f"Send error: {e}")
                return False
        self._debug("Send failed: not connected")
        return False

    def _receive_data(self):
        buffer = bytearray()
        last_debug_time = time.time()
        bytes_received = 0

        self._debug("Receiver thread started, waiting for data...")

        while not self.stop_thread and self.is_connected:
            try:
                if self.serial_port.in_waiting > 0:
                    data = self.serial_port.read(self.serial_port.in_waiting)
                    bytes_received += len(data)
                    buffer.extend(data)
                    self.buffer_size = len(buffer)

                    current_time = time.time()
                    if current_time - last_debug_time > 2.0:
                        self._debug(f"Buffer size: {len(buffer)} bytes, Total received: {bytes_received} bytes")
                        last_debug_time = current_time

                    processed = 0
                    while len(buffer) > 0:
                        if len(buffer) >= 8 and buffer[:4] == b'rsp:':
                            response = buffer[:8].decode('ascii')
                            buffer = buffer[8:]
                            processed += 8
                            self._debug(f"Response received: {response}")
                            if self.receive_callback:
                                self.receive_callback('response', response)
                            continue

                        if len(buffer) >= 4 and buffer[:4] == b'jpg:':
                            self._debug(f"Found jpg: header at position 0")
                            if len(buffer) >= 13:
                                fps = buffer[4]
                                frame_num = struct.unpack('<I', buffer[5:9])[0]
                                data_size = struct.unpack('<I', buffer[9:13])[0]

                                self._debug(f"FPS: {fps}, Frame #{frame_num}, Size: {data_size} bytes")

                                if data_size > 1000000:
                                    self._debug(f"Invalid data size: {data_size}, skipping 4 bytes")
                                    buffer = buffer[4:]
                                    continue

                                total_size = 13 + data_size
                                if len(buffer) >= total_size:
                                    jpeg_data = buffer[13:13 + data_size - 4]

                                    end_marker_bytes = buffer[13 + data_size - 4:13 + data_size]
                                    end_marker = struct.unpack('<I', end_marker_bytes)[0]

                                    if end_marker == END_MARKER_VALUE:
                                        self._debug(f"End marker OK (0x{end_marker:08X}), JPEG frame #{frame_num} extracted ({len(jpeg_data)} bytes)")
                                        processed += total_size
                                        if self.receive_callback:
                                            self.receive_callback('jpeg', {
                                                'frame_num': frame_num,
                                                'fps': fps,
                                                'data': jpeg_data
                                            })
                                    else:
                                        self._debug(f"Invalid end marker: 0x{end_marker:08X}, expected: 0x{END_MARKER_VALUE:08X}")

                                    buffer = buffer[total_size:]
                                    continue
                                else:
                                    self._debug(f"Incomplete frame: have {len(buffer)}/{total_size} bytes")
                                    break
                            else:
                                self._debug("Incomplete header, waiting for more data")
                                break

                        buffer = buffer[1:]
                        processed += 1

                    if processed > 0:
                        self._debug(f"Processed {processed} bytes in this iteration")

                time.sleep(0.001)
            except Exception as e:
                self._debug(f"Receive error: {e}")
                time.sleep(1)

    def set_receive_callback(self, callback):
        self.receive_callback = callback


class ProtocolHandler:
    def __init__(self, serial_comm):
        self.serial_comm = serial_comm
        self.response_callback = None
        self.frame_callback = None
        self.debug_callback = None
        self.frames_received = 0

    def set_debug_callback(self, callback):
        self.debug_callback = callback
        self.serial_comm.set_debug_callback(callback)

    def _debug(self, message):
        if self.debug_callback:
            self.debug_callback(f"[PROTO] {message}")

    def set_callbacks(self, response_callback, frame_callback):
        self.response_callback = response_callback
        self.frame_callback = frame_callback
        self.serial_comm.set_receive_callback(self._on_data_received)

    def _on_data_received(self, data_type, data):
        if data_type == 'response' and self.response_callback:
            self._debug(f"Forwarding response: {data}")
            self.response_callback(data)
        elif data_type == 'jpeg' and self.frame_callback:
            self.frames_received += 1
            self._debug(f"Forwarding JPEG frame #{data['frame_num']} ({self.frames_received} frames total)")
            self.frame_callback(data)

    def start_recording_qvga(self):
        self._debug("Sending QVGA recording command")
        return self.serial_comm.send_data('a')

    def start_recording_vga(self):
        self._debug("Sending VGA recording command")
        return self.serial_comm.send_data('b')

    def start_recording_hd(self):
        self._debug("Sending HD recording command")
        return self.serial_comm.send_data('c')

    def stop_recording(self):
        self._debug("Sending stop recording command")
        return self.serial_comm.send_data('0')

    def take_snapshot(self):
        self._debug("Sending snapshot command")
        return self.serial_comm.send_data('s')

    def shutdown_device(self):
        self._debug("Sending shutdown command")
        return self.serial_comm.send_data('1')

    def set_white_balance(self, mode):
        self._debug(f"Sending white balance command: {mode}")
        return self.serial_comm.send_data(mode)


class VideoSaver:
    def __init__(self, output_dir='videos', photos_dir='photos'):
        self.output_dir = output_dir
        self.photos_dir = photos_dir
        self.video_writer = None
        self.current_filename = None
        self.frame_count = 0
        self.debug_callback = None
        self.current_fps = None
        self.current_frame_size = None
        self.is_initialized = False

        os.makedirs(output_dir, exist_ok=True)
        os.makedirs(photos_dir, exist_ok=True)

    def set_debug_callback(self, callback):
        self.debug_callback = callback

    def _debug(self, message):
        if self.debug_callback:
            self.debug_callback(f"[VIDEO] {message}")

    def start_new_video(self):
        self.close_video()
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        self.current_filename = os.path.join(self.output_dir, f"video_{timestamp}.mp4")
        self._debug(f"Creating new video file: {self.current_filename}")
        self.frame_count = 0
        self.is_initialized = False
        return self.current_filename

    def _initialize_video_writer(self, frame_size, fps):
        self._debug(f"Initializing video writer with size: {frame_size}, FPS: {fps}")

        fourcc = cv2.VideoWriter_fourcc(*'mp4v')
        self.video_writer = cv2.VideoWriter(
            self.current_filename,
            fourcc,
            fps,
            frame_size
        )

        if not self.video_writer.isOpened():
            self._debug("ERROR: Failed to open video writer")
            return False

        self._debug("Video writer initialized successfully")
        self.current_fps = fps
        self.current_frame_size = frame_size
        self.is_initialized = True
        return True

    def save_snapshot(self, jpeg_data):
        try:
            img_array = np.frombuffer(jpeg_data, dtype=np.uint8)
            frame = cv2.imdecode(img_array, cv2.IMREAD_COLOR)

            if frame is not None:
                self._debug(f"Snapshot decoded: {frame.shape}")
                timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                snapshot_filename = os.path.join(self.photos_dir, f"snapshot_{timestamp}.jpg")

                result = cv2.imwrite(snapshot_filename, frame)
                if result:
                    self._debug(f"Snapshot saved to: {snapshot_filename}")
                    return snapshot_filename
                else:
                    self._debug("ERROR: Failed to save snapshot")
                    return None
            else:
                self._debug("ERROR: Failed to decode snapshot JPEG data")
                return None
        except Exception as e:
            self._debug(f"Error saving snapshot: {e}")
            return None

    def add_frame(self, jpeg_data, fps=None):
        try:
            img_array = np.frombuffer(jpeg_data, dtype=np.uint8)
            frame = cv2.imdecode(img_array, cv2.IMREAD_COLOR)

            if frame is not None:
                if not self.is_initialized:
                    frame_size = (frame.shape[1], frame.shape[0])
                    if fps is None:
                        self._debug("ERROR: FPS not provided for initialization")
                        return False

                    if not self._initialize_video_writer(frame_size, fps):
                        return False

                self._debug(f"Frame decoded: {frame.shape}")
                self.video_writer.write(frame)
                self.frame_count += 1

                if self.frame_count % 10 == 0:
                    self._debug(f"Processed {self.frame_count} frames so far")

                return True
            else:
                self._debug("ERROR: Failed to decode JPEG data")
                return False
        except Exception as e:
            self._debug(f"Error adding frame: {e}")
            return False

    def close_video(self):
        if self.video_writer is not None:
            self._debug(f"Closing video file, processed {self.frame_count} frames")
            self.video_writer.release()
            self.video_writer = None
            self.is_initialized = False
            self.current_fps = None
            self.current_frame_size = None
            return self.current_filename, self.frame_count
        return None, 0


class AppController:
    def __init__(self, com_port):
        self.state = RecordingState.IDLE
        self.state_text = "IDLE"

        self.serial_comm = SerialCommunication(com_port)
        self.protocol = ProtocolHandler(self.serial_comm)
        self.video_saver = VideoSaver()

        self.status_callback = None
        self.debug_callback = None
        self.frame_listener = None

    def set_status_callback(self, callback):
        self.status_callback = callback

    def set_debug_callback(self, callback):
        self.debug_callback = callback
        self.protocol.set_debug_callback(callback)
        self.video_saver.set_debug_callback(callback)

    def set_frame_listener(self, callback):
        self.frame_listener = callback

    def clear_frame_listener(self):
        self.frame_listener = None

    def _debug(self, message):
        if self.debug_callback:
            self.debug_callback(f"[APP] {message}")

    def _notify_status(self, message):
        self._update_state_text()
        if self.status_callback:
            self.status_callback(message)

    def _update_state_text(self):
        if self.state == RecordingState.IDLE:
            self.state_text = "IDLE"
        else:
            fps_info = f", FPS: {self.video_saver.current_fps}" if self.video_saver.current_fps else ""
            self.state_text = f"RECORDING (Frames: {self.video_saver.frame_count}{fps_info})"

    def connect(self):
        self._debug("Connecting to device...")
        self.protocol.set_callbacks(
            self._on_response_received,
            self._on_frame_received
        )

        result = self.serial_comm.connect()
        if result:
            self._notify_status("Connected to device")
        else:
            self._notify_status("Failed to connect to device")
        return result

    def disconnect(self):
        self._debug("Disconnecting from device...")
        if self.state == RecordingState.RECORDING:
            self.stop_recording()
        self.serial_comm.disconnect()
        self._notify_status("Disconnected from device")

    def set_white_balance(self, mode):
        if isinstance(mode, WhiteBalanceMode):
            mode = mode.value
        self._debug(f"Setting white balance to: {mode}")
        self.protocol.set_white_balance(mode)
        self._notify_status(f"White balance set to: {mode}")

    def start_recording_qvga(self):
        if self.state == RecordingState.IDLE:
            self._debug("Starting QVGA recording...")
            self._notify_status("Starting QVGA recording...")
            self.protocol.start_recording_qvga()
        else:
            self._debug("Already recording")
            self._notify_status("Already recording")

    def start_recording_vga(self):
        if self.state == RecordingState.IDLE:
            self._debug("Starting VGA recording...")
            self._notify_status("Starting VGA recording...")
            self.protocol.start_recording_vga()
        else:
            self._debug("Already recording")
            self._notify_status("Already recording")

    def start_recording_hd(self):
        if self.state == RecordingState.IDLE:
            self._debug("Starting HD recording...")
            self._notify_status("Starting HD recording...")
            self.protocol.start_recording_hd()
        else:
            self._debug("Already recording")
            self._notify_status("Already recording")

    def stop_recording(self):
        if self.state == RecordingState.RECORDING:
            self._debug("Stopping recording...")
            self._notify_status("Stopping recording...")
            self.protocol.stop_recording()
        else:
            self._debug("Not recording")
            self._notify_status("Not recording")

    def take_snapshot(self):
        if self.state == RecordingState.IDLE:
            self._debug("Taking snapshot...")
            self._notify_status("Taking snapshot...")
            self.protocol.take_snapshot()
        else:
            self._debug("Cannot take snapshot while recording")
            self._notify_status("Cannot take snapshot while recording")

    def shutdown_device(self):
        self._debug("Shutting down device...")
        self._notify_status("Shutting down device...")
        self.protocol.shutdown_device()

    def _on_response_received(self, response):
        self._debug(f"Received response: {response}")

        if response == ResponseCode.START_SUCCESS.value:
            self.state = RecordingState.RECORDING
            filename = self.video_saver.start_new_video()
            self._update_state_text()
            self._notify_status(f"Recording started. Saving to {filename}")

        elif response == ResponseCode.START_FAILURE.value:
            self._notify_status("Failed to start recording")

        elif response == ResponseCode.ALREADY_RECORDING.value:
            self._notify_status("Device is already recording")

        elif response == ResponseCode.STOP_SUCCESS.value:
            self.state = RecordingState.IDLE
            filename, frame_count = self.video_saver.close_video()
            self._update_state_text()
            self._notify_status(f"Recording stopped. Saved {frame_count} frames to {filename}")

        elif response == ResponseCode.STOP_FAILURE.value:
            self._notify_status("Failed to stop recording")

        elif response == ResponseCode.NOT_RECORDING.value:
            self._notify_status("Device is not recording")

        elif response == ResponseCode.SNAPSHOT_SUCCESS.value:
            self._notify_status("Snapshot taken successfully")

        elif response == ResponseCode.SNAPSHOT_FAILURE.value:
            self._notify_status("Failed to take snapshot")

    def _on_frame_received(self, frame_data):
        frame_num = frame_data['frame_num']
        fps = frame_data['fps']
        jpeg_data = frame_data['data']

        if frame_num != SNAPSHOT_FRAME_NUMBER:
            self._update_state_text()
            if self.state == RecordingState.RECORDING:
                self.video_saver.add_frame(jpeg_data, fps)
            else:
                self._debug(f"Received frame #{frame_num} but not recording")

            if self.frame_listener:
                self.frame_listener(jpeg_data)
        else:
            self._debug(f"Received snapshot frame")
            snapshot_path = self.video_saver.save_snapshot(jpeg_data)
            if snapshot_path:
                self._notify_status(f"Snapshot saved to {snapshot_path}")
            else:
                self._notify_status("Failed to save snapshot")


class ConsoleUI:
    def __init__(self):
        self.controller = None
        self.running = False

    def set_controller(self, controller):
        self.controller = controller
        self.controller.set_status_callback(self._on_status_update)
        self.controller.set_debug_callback(self._on_debug_message)

    def _on_status_update(self, message):
        state_info = f"[STATE: {self.controller.state_text}] "
        print(f"\n[STATUS] {state_info}{message}")

    def _on_debug_message(self, message):
        print(f"[DEBUG] {message}")

    def _print_menu(self):
        state_info = f"[STATE: {self.controller.state_text}] "

        print(f"\n=== Video Recording Control {state_info}===")
        print("1. Start Recording (QVGA)")
        print("2. Start Recording (VGA)")
        print("3. Start Recording (HD)")
        print("4. Stop Recording")
        print("5. Take Snapshot (only when not recording)")
        print("6. Set White Balance")
        print("7. Shutdown Device")
        print("8. Exit Program")
        print("Enter choice (1-8): ", end="")

    def _show_white_balance_menu(self):
        print("\n=== White Balance Settings ===")
        for i, mode in enumerate(WhiteBalanceMode, 1):
            print(f"{i}. {mode.name}")
        print("0. Cancel")
        print("Enter choice (0-6): ", end="")

        try:
            choice = int(input().strip())
            if 0 < choice <= len(WhiteBalanceMode):
                mode = list(WhiteBalanceMode)[choice - 1]
                self.controller.set_white_balance(mode)
            elif choice == 0:
                print("Cancelled")
            else:
                print("Invalid choice")
        except ValueError:
            print("Invalid input")

    def start(self, com_port):
        self.controller = AppController(com_port)

        if not self.controller.connect():
            print("Failed to connect to device. Exiting.")
            return

        self.running = True
        while self.running:
            try:
                self._print_menu()
                choice = input().strip()

                if choice == '1':
                    self.controller.start_recording_qvga()
                elif choice == '2':
                    self.controller.start_recording_vga()
                elif choice == '3':
                    self.controller.start_recording_hd()
                elif choice == '4':
                    self.controller.stop_recording()
                elif choice == '5':
                    self.controller.take_snapshot()
                elif choice == '6':
                    self._show_white_balance_menu()
                elif choice == '7':
                    self.controller.shutdown_device()
                elif choice == '8':
                    self.running = False
                    print("Exiting...")
                else:
                    print("Invalid choice. Please try again.")

            except KeyboardInterrupt:
                self.running = False
                print("\nExiting...")

        self.controller.disconnect()


def cli_main():
    print("=== Video Recording Application ===")
    com_port = input("Enter COM port (e.g. COM3 or /dev/ttyUSB0): ")

    ui = ConsoleUI()
    ui.start(com_port)


class WebUI:
    def __init__(self):
        self.app = Flask(__name__)
        self.controller = None
        self.frame_queue = queue.Queue(maxsize=30)
        self.status_messages = []

        self._setup_routes()

    def _setup_routes(self):
        @self.app.route('/')
        def index():
            html = '''
<!DOCTYPE html>
<html>
<head>
    <title>Video Device Control</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { display: flex; gap: 20px; }
        .controls { flex: 1; max-width: 400px; }
        .display { flex: 2; }
        .section { margin-bottom: 20px; }

        button {
            margin: 5px;
            padding: 6px 12px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
        }

        .start-button {
            background-color: #4CAF50;
            color: white;
        }
        .start-button:hover {
            background-color: #45a049;
        }

        .stop-button {
            background-color: #f44336;
            color: white;
        }
        .stop-button:hover {
            background-color: #da190b;
        }

        #status {
            margin-top: 20px;
            padding: 10px;
            border: 1px solid #ccc;
            height: 200px;
            overflow-y: scroll;
        }

        #video {
            margin-top: 20px;
            border: 1px solid #ccc;
            max-width: 100%;
        }

        input, select {
            padding: 6px;
            margin: 5px;
        }
.guide {
    background-color: #f8f9fa;
    padding: 15px;
    border-radius: 4px;
    font-size: 14px;
}

.guide ol {
    margin-left: 20px;
}

.guide li {
    margin-bottom: 10px;
}

.guide ul {
    margin-top: 5px;
    margin-left: 20px;
}

.guide ul li {
    margin-bottom: 5px;
}
    </style>
</head>
<body>

    <div class="container">
        <div class="controls">
            <div class="section">
                <h3>シリアルポート</h3>
                <input type="text" id="comPort" placeholder="COM3 or /dev/ttyUSB0">
                <button onclick="connect()" class="start-button">接続</button>
                <button onclick="disconnect()" class="stop-button">切断</button>
            </div>

            <div class="section">
                <h3>動画</h3>
                <button onclick="startRecording('qvga')" class="start-button">開始(QVGA)</button>
                <button onclick="startRecording('vga')" class="start-button">開始(VGA)</button>
                <button onclick="startRecording('hd')" class="start-button">開始(HD)</button>
                <button onclick="stopRecording()" class="stop-button">停止</button>
            </div>

            <div class="section">
                <h3>その他</h3>
                <button onclick="takeSnapshot()" class="start-button">静止画撮影</button>
                <button onclick="shutdownDevice()" class="stop-button">シャットダウン</button>
            </div>

            <div class="section">
                <h3>ホワイトバランス</h3>
                    <select id="whiteBalance">
                        <option value="B">自動</option>
                        <option value="C">白熱灯</option>
                        <option value="D">蛍光灯</option>
                        <option value="G">昼光</option>
                        <option value="I">曇り</option>
                        <option value="J">日陰</option>
                    </select>
                <button onclick="setWhiteBalance()" class="start-button">設定</button>
            </div>
<div class="section">
    <h3>使い方ガイド</h3>
    <div class="guide">
        <ol>
            <li><strong>接続方法</strong><br>
                シリアルポート欄にCOMポート番号（例：COM3）を入力し、「接続」ボタンをクリックしてください。
            </li>
            <li><strong>動画撮影</strong>
                <ul>
                    <li>撮影を開始するには、解像度（QVGA、VGA、HD）ごとの「開始」ボタンをクリックします。</li>
                    <li>撮影を終了する際は、必ず「停止」ボタンをクリックしてください。</li>
                    <li>現在の映像は「Live Video」エリアにリアルタイムで表示されます。</li>
                </ul>
            </li>
            <li><strong>静止画撮影（FullHD）</strong><br>
                静止画を撮影する際は、動画録画が停止している状態でのみ実行可能です。
            </li>
            <li><strong>ホワイトバランス設定</strong><br>
                ホワイトバランスを調整する際も、動画録画を停止してから設定してください。
            </li>
        </ol>
    </div>
</div>

        </div>

        <div class="display">
            <div class="section">
                <h3>Live Video</h3>
                <img id="video" src="/video_feed" width="640" height="480">
            </div>

            <div class="section">
                <h3>Status</h3>
                <div id="status"></div>
            </div>
        </div>
    </div>

    <script>
        function updateStatus() {
            fetch('/status')
                .then(response => response.json())
                .then(data => {
                    document.getElementById('status').innerHTML = data.messages.join('<br>');
                });
        }

        function connect() {
            const comPort = document.getElementById('comPort').value;
            fetch('/connect', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({port: comPort})
            }).then(updateStatus);
        }

        function disconnect() {
            fetch('/disconnect', {method: 'POST'}).then(updateStatus);
        }

        function startRecording(mode) {
            fetch(`/record/${mode}`, {method: 'POST'}).then(updateStatus);
        }

        function stopRecording() {
            fetch('/record/stop', {method: 'POST'}).then(updateStatus);
        }

        function takeSnapshot() {
            fetch('/snapshot', {method: 'POST'}).then(updateStatus);
        }

        function shutdownDevice() {
            fetch('/shutdown', {method: 'POST'}).then(updateStatus);
        }

        function setWhiteBalance() {
            const mode = document.getElementById('whiteBalance').value;
            fetch(`/white_balance/${mode}`, {method: 'POST'}).then(updateStatus);
        }

        setInterval(updateStatus, 1000);
    </script>
</body>
</html>
            '''
            return render_template_string(html)

        @self.app.route('/connect', methods=['POST'])
        def connect():
            port = request.json.get('port')
            if not port:
                return jsonify({'error': 'Port not specified'}), 400

            self.controller = AppController(port)
            self.controller.set_status_callback(self._status_callback)
            self.controller.set_frame_listener(self._frame_callback)

            if self.controller.connect():
                return jsonify({'status': 'connected'})
            else:
                return jsonify({'error': 'Failed to connect'}), 500

        @self.app.route('/disconnect', methods=['POST'])
        def disconnect():
            if self.controller:
                self.controller.disconnect()
                return jsonify({'status': 'disconnected'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/record/qvga', methods=['POST'])
        def record_qvga():
            if self.controller:
                self.controller.start_recording_qvga()
                return jsonify({'status': 'recording QVGA'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/record/vga', methods=['POST'])
        def record_vga():
            if self.controller:
                self.controller.start_recording_vga()
                return jsonify({'status': 'recording VGA'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/record/hd', methods=['POST'])
        def record_hd():
            if self.controller:
                self.controller.start_recording_hd()
                return jsonify({'status': 'recording HD'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/record/stop', methods=['POST'])
        def stop_recording():
            if self.controller:
                self.controller.stop_recording()
                return jsonify({'status': 'stopped'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/snapshot', methods=['POST'])
        def snapshot():
            if self.controller:
                self.controller.take_snapshot()
                return jsonify({'status': 'snapshot taken'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/white_balance/<mode>', methods=['POST'])
        def white_balance(mode):
            if self.controller:
                self.controller.set_white_balance(mode)
                return jsonify({'status': f'white balance set to {mode}'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/shutdown', methods=['POST'])
        def shutdown():
            if self.controller:
                self.controller.shutdown_device()
                return jsonify({'status': 'shutdown command sent'})
            return jsonify({'error': 'Not connected'}), 400

        @self.app.route('/status', methods=['GET'])
        def status():
            return jsonify({'messages': self.status_messages[-10:]})

        @self.app.route('/video_feed')
        def video_feed():
            return Response(self._generate_frames(),
                          mimetype='multipart/x-mixed-replace; boundary=frame')

    def _status_callback(self, message):
        self.status_messages.append(message)
        if len(self.status_messages) > 50:
            self.status_messages.pop(0)

    def _frame_callback(self, jpeg_data):
        try:
            if self.frame_queue.full():
                self.frame_queue.get_nowait()
            self.frame_queue.put_nowait(jpeg_data)
        except:
            pass

    def _generate_frames(self):
        while True:
            try:
                frame = self.frame_queue.get(timeout=1.0)
                yield (b'--frame\r\n'
                       b'Content-Type: image/jpeg\r\n\r\n' + frame + b'\r\n')
            except queue.Empty:
                pass

    def run(self, host='0.0.0.0', port=5000):
        self.app.run(host=host, port=port, threaded=True)

def web_main():
    import argparse
    import logging
    import sys

    logging.getLogger('werkzeug').setLevel(logging.ERROR)

    parser = argparse.ArgumentParser(description='Video Recording Web Interface')
    parser.add_argument('--port', type=int, default=5100, help='Port number (default: 5100)')
    args = parser.parse_args()

    print("=== Video Recording Web Interface ===")
    ui = WebUI()

    ui.app.logger.setLevel(logging.ERROR)

    log = logging.getLogger('werkzeug')
    log.setLevel(logging.ERROR)

    cli = sys.modules['flask.cli']
    cli.show_server_banner = lambda *x: None

    print(f"Starting web server on http://localhost:{args.port}")
    ui.run(host='0.0.0.0', port=args.port)


if __name__ == "__main__":
    # cli_main()
    web_main()
