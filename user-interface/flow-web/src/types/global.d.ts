declare global {
    interface Window {
        electronAPI?: {
            getAppVersion: () => Promise<string>;
            getPlatform: () => Promise<string>;
        };
        environment?: {
            isElectron: boolean;
            platform: string;
            electronVersion: string;
            nodeVersion: string;
            chromeVersion: string;
        };
    }

    interface PublicKeyCredentialCreationOptions {
        challenge: ArrayBuffer;
        rp: {
            name: string;
            id?: string;
        };
        user: {
            id: ArrayBuffer;
            name: string;
            displayName: string;
        };
        pubKeyCredParams: Array<{
            type: string;
            alg: number;
        }>;
        timeout?: number;
        excludeCredentials?: Array<{
            id: ArrayBuffer;
            type: string;
            transports?: string[];
        }>;
        authenticatorSelection?: {
            authenticatorAttachment?: string;
            requireResidentKey?: boolean;
            residentKey?: string;
            userVerification?: string;
        };
        attestation?: string;
    }
}

export { };
