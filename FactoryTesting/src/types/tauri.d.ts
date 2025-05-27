// Tauri 类型声明
declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: (cmd: string, args?: any) => Promise<any>;
      };
      event: {
        listen: (event: string, handler: (event: any) => void) => Promise<() => void>;
        emit: (event: string, payload?: any) => Promise<void>;
      };
      fs: any;
      path: any;
      dialog: any;
      shell: any;
      app: any;
      window: any;
      clipboard: any;
      globalShortcut: any;
      notification: any;
      process: any;
      updater: any;
    };
  }
}

export {}; 