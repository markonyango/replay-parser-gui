// Specs run in a plain browser, so stub the Tauri runtime before any module
// calls getCurrentWebviewWindow() at import time.
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';

mockWindows('main');
mockIPC(
    (cmd) => {
        switch (cmd) {
            case 'get_status':
                return { text: '', indicators: [] };
            case 'plugin:store|load':
                return 1;
            case 'plugin:store|get':
                return [undefined, false];
            default:
                return undefined;
        }
    },
    { shouldMockEvents: true }
);
