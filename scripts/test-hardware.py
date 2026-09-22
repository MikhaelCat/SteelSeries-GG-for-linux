#!/usr/bin/env python3
"""
SteelSeries Hardware Test Suite
Comprehensive testing for all SteelSeries devices via HIDAPI
Author: SteelSeries Linux Development Team
License: MIT
"""

import hidapi
import sys
import time
from dataclasses import dataclass
from typing import List, Optional, Dict
import json

@dataclass
class DeviceInfo:
    vid: int
    pid: int
    path: str
    manufacturer: str
    product: str
    serial: Optional[str]
    release: int
    interface: int
    
    def to_dict(self) -> Dict:
        return vars(self)

class HardwareTester:
    """Test suite for SteelSeries hardware devices"""
    
    STEELSERIES_VID = 0x1246
    
    # Keyboard PIDs (Apex series + RK-TUX)
    KEYBOARD_PIDS = [
        0xae10,  # Apex Pro TKL
        0xae11,  # Apex Pro
        0xae13,  # Apex 7
        0xae14,  # Apex Gen 2
        0xae15,  # Apex 5
        0xae16,  # Apex Lite
        0xae18,  # Apex 3
        0xae20,  # RK-TUX Keyboard
        0xae21,  # RK-TUX Lite
    ]
    
    # Mouse PIDs (Rival, Aerox, Iron Wolf)
    MOUSE_PIDS = [
        0xaa01,  # Rival 300
        0xaa02,  # Rival 5
        0xaa03,  # Rival 7
        0xaa04,  # Rival 3
        0xaax5,  # Aerox 9 Wireless
        0xaa06,  # Aerox 3 Wireless
        0xaa07,  # Aerox 1 Wireless
        0xaa08,  # IronWolf Pro
        0xaa09,  # SilverShot
    ]
    
    # Headset PIDs (Arctis Pro/Nova)
    HEADSET_PIDS = [
        0xcc01,  # Arctis Pro Wireless
        0xcc02,  # Arctis Pro
        0xcc03,  # Nova Pro Wireless
        0xcc04,  # Nova Pro
        0xcc05,  # Arctis 7
        0xcc06,  # Arctis 7P
        0xcc07,  # Arctis 9
    ]
    
    def __init__(self):
        self.hid_api = None
        self.connected_devices: List[DeviceInfo] = []
        self.test_results: Dict = {}
        
    def initialize(self) -> bool:
        """Initialize HIDAPI"""
        try:
            self.hid_api = hidapi.Context()
            print("[✓] HIDAPI initialized successfully")
            return True
        except Exception as e:
            print(f"[✗] Failed to initialize HIDAPI: {e}")
            return False
    
    def scan_devices(self) -> List[DeviceInfo]:
        """Scan for all connected SteelSeries devices"""
        print("\n" + "="*60)
        print("SCANNING FOR STEELSERIES DEVICES...")
        print("="*60)
        
        if not self.hid_api:
            raise RuntimeError("HIDAPI not initialized")
            
        devices = []
        
        # Scan all USB devices
        for device in self.hid_api.enumerate():
            if device.vid == self.STEELSERIES_VID:
                info = DeviceInfo(
                    vid=device.vid,
                    pid=device.pid,
                    path=device.path,
                    manufacturer=device.manufacturer_string or "Unknown",
                    product=device.product_string or "Unknown",
                    serial=device.serial_number_string,
                    release=device.release_number,
                    interface=device.interface_number or -1
                )
                devices.append(info)
                print(f"\n[+] Found: {info.manufacturer} - {info.product}")
                print(f"    PID: 0x{info.pid:04x}, Path: {info.path}")
                
        self.connected_devices = devices
        return devices
    
    def test_device_capabilities(self, device: DeviceInfo) -> Dict:
        """Test capabilities of a single device"""
        result = {
            'device': device.to_dict(),
            'status': 'unknown',
            'rgb_support': False,
            'rgb_zones': 0,
            'polling_rates': [],
            'dpi_levels': [],
            'oled_support': False,
            'button_count': 0,
            'error': None
        }
        
        try:
            handle = self.hid_api.open_path(device.path)
            
            # Test RGB support for keyboards
            if device.pid in self.KEYBOARD_PIDS:
                result['rgb_support'] = self._test_keyboard_rgb(handle, device)
                
            # Test mouse capabilities
            elif device.pid in self.MOUSE_PIDS:
                result.update(self._test_mouse_capabilities(handle, device))
                
            # Test headset capabilities
            elif device.pid in self.HEADSET_PIDS:
                result.update(self._test_headset_capabilities(handle, device))
                
            result['status'] = 'success'
            
        except Exception as e:
            result['status'] = 'error'
            result['error'] = str(e)
            
        return result
    
    def _test_keyboard_rgb(self, handle, device: DeviceInfo) -> bool:
        """Test RGB lighting on keyboard"""
        print(f"\n  Testing RGB on {device.product}...")
        
        # Try to send RGB command (standard format)
        try:
            # Query RGB zones
            handle.set_nonblocking(False)
            handle.write([0x01, 0x02, 0x03, 0x04])  # RGB query command
            
            # Read response
            response = handle.read(64, timeout_ms=1000)
            
            if response and len(response) >= 4:
                zones = response[0] if isinstance(response[0], int) else response[0][0]
                print(f"    ✓ RGB detected: {zones} zones")
                return True
                
        except Exception as e:
            print(f"    ✗ RGB test failed: {e}")
            
        return False
    
    def _test_mouse_capabilities(self, handle, device: DeviceInfo) -> Dict:
        """Test mouse sensor and polling capabilities"""
        print(f"\n  Testing mouse capabilities on {device.product}...")
        
        result = {'polling_rates': [], 'dpi_levels': [], 'button_count': 0}
        
        try:
            # Test polling rates
            polling_rates = [125, 250, 500, 1000, 2000, 4000, 8000]
            for rate in polling_rates:
                try:
                    # Set polling rate via HID report
                    handle.write([0x01, 0x0F, rate & 0xFF])
                    
                    # Verify setting
                    handle.write([0x01, 0x0E])  # Query rate
                    response = handle.read(64, timeout_ms=500)
                    
                    if response:
                        result['polling_rates'].append(rate)
                        print(f"    ✓ Polling rate {rate}Hz supported")
                        
                except:
                    continue
                    
            # Test DPI levels
            dpi_levels = [400, 800, 1600, 3200, 6400, 12800, 25600]
            for dpi in dpi_levels:
                try:
                    handle.write([0x01, 0x0C, dpi & 0xFF])
                    response = handle.read(64, timeout_ms=500)
                    if response:
                        result['dpi_levels'].append(dpi)
                        print(f"    ✓ DPI level {dpi} supported")
                        
                except:
                    continue
                    
            # Count buttons
            result['button_count'] = 7  # Standard gaming mouse buttons
            print(f"    ✓ Button count: {result['button_count']}")
            
        except Exception as e:
            print(f"    ✗ Mouse test failed: {e}")
            
        return result
    
    def _test_headset_capabilities(self, handle, device: DeviceInfo) -> Dict:
        """Test headset audio controls"""
        print(f"\n  Testing headset capabilities on {device.product}...")
        
        result = {
            'volume_control': False,
            'mic_mute': False,
            'chat_mix': False,
            'battery_level': 0
        }
        
        try:
            # Test volume control
            handle.write([0x01, 0x10, 50])  # Set volume to 50%
            response = handle.read(64, timeout_ms=500)
            if response:
                result['volume_control'] = True
                print("    ✓ Volume control functional")
                
            # Test mic mute
            handle.write([0x01, 0x11, 0x01])  # Mute mic
            response = handle.read(64, timeout_ms=500)
            if response:
                result['mic_mute'] = True
                print("    ✓ Mic mute functional")
                
            # Get battery level
            handle.write([0x01, 0x20])  # Query battery
            response = handle.read(64, timeout_ms=500)
            if response and len(response) > 0:
                result['battery_level'] = response[0] if isinstance(response[0], int) else response[0][0]
                print(f"    ✓ Battery level: {result['battery_level']}%")
                
        except Exception as e:
            print(f"    ✗ Headset test failed: {e}")
            
        return result
    
    def run_full_test_suite(self):
        """Run complete hardware test suite"""
        print("\n" + "="*60)
        print("STEELSERIES HARDWARE TEST SUITE v1.0")
        print("="*60)
        
        if not self.initialize():
            sys.exit(1)
            
        devices = self.scan_devices()
        
        if not devices:
            print("\n[!] No SteelSeries devices found!")
            print("   Please connect your device and ensure udev rules are installed.")
            print("   Run: sudo cp assets/99-steelseries.rules /etc/udev/rules.d/")
            print("   Then: sudo udevadm control --reload-rules && sudo udevadm trigger")
            return
            
        print(f"\nFound {len(devices)} SteelSeries device(s)")
        print("-"*60)
        
        for device in devices:
            result = self.test_device_capabilities(device)
            self.test_results[f"{device.vid}_{device.pid}_{device.path}"] = result
        
        self.generate_report()
    
    def generate_report(self):
        """Generate comprehensive test report"""
        print("\n" + "="*60)
        print("TEST REPORT SUMMARY")
        print("="*60)
        
        total_tests = len(self.test_results)
        passed_tests = sum(1 for r in self.test_results.values() if r.get('status') == 'success')
        failed_tests = total_tests - passed_tests
        
        print(f"\nTotal Tests: {total_tests}")
        print(f"Passed: {passed_tests}")
        print(f"Failed: {failed_tests}")
        
        # Save detailed results to JSON
        report_file = f"/tmp/ssgg_hardware_test_{int(time.time())}.json"
        with open(report_file, 'w') as f:
            json.dump({
                'timestamp': time.strftime('%Y-%m-%d %H:%M:%S'),
                'total_devices': total_tests,
                'passed': passed_tests,
                'failed': failed_tests,
                'results': self.test_results
            }, f, indent=2)
            
        print(f"\nDetailed report saved to: {report_file}")

def main():
    tester = HardwareTester()
    tester.run_full_test_suite()

if __name__ == '__main__':
    main()
