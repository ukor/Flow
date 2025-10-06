import { useState } from "react";
import "./App.css";
import FlowLogo from "./components/FlowLogo";
import ElectronInfo from "./components/ElectronInfo";


function App() {
  const [count, setCount] = useState(0);
  const [successMessage, setSuccessMessage] = useState('')
  const [errorMessage, setErrorMessage] = useState('')
  const [isRegistering, setIsRegistering] = useState(false)
  const [isAuthenticating, setIsAuthenticating] = useState(false)


  const handleRegistration = async () => {
    // Reset success/error messages
    setSuccessMessage('')
    setErrorMessage('')
    setIsRegistering(true)

    try {
      // GET registration options from the endpoint that calls
      // @simplewebauthn/server -> generateRegistrationOptions()
      const resp = await fetch('http://localhost:8080/api/v1/webauthn/start_registration')

      if (!resp.ok) {
        throw new Error(`HTTP error! status: ${resp.status}`)
      }

      const options  = await resp.json();
      const challenge_id = options.publicKey.challenge;

      const publicKeyCredentialCreationOptions: PublicKeyCredentialCreationOptions = {
        challenge: base64UrlToBuffer(options.publicKey.challenge),
        rp: options.publicKey.rp,
        user: {
          id: base64UrlToBuffer(options.publicKey.user.id),
          name: options.publicKey.user.name,
          displayName: options.publicKey.user.displayName,
        },
        pubKeyCredParams: options.publicKey.pubKeyCredParams,
        timeout: options.publicKey.timeout,
        excludeCredentials: options.publicKey.excludeCredentials?.map((cred: any) => ({
          id: base64UrlToBuffer(cred.id),
          type: cred.type,
          transports: cred.transports,
        })),
        authenticatorSelection: options.publicKey.authenticatorSelection,
        attestation: options.publicKey.attestation,
      };

      // 3. Call native browser WebAuthn API
      const credential = await navigator.credentials.create({
        publicKey: publicKeyCredentialCreationOptions,
      }) as PublicKeyCredential;

      if (!credential) {
        throw new Error('Failed to create credential');
      }

      // 4. Convert credential response to format for server
      const attestationResponse = credential.response as AuthenticatorAttestationResponse;
      const credentialJSON = {
        id: credential.id,
        rawId: bufferToBase64Url(credential.rawId),
        response: {
          attestationObject: bufferToBase64Url(attestationResponse.attestationObject),
          clientDataJSON: bufferToBase64Url(attestationResponse.clientDataJSON),
        },
        type: credential.type,
        extentions: credential.getClientExtensionResults(),
      };

      // 5. Send to server for verification
      const verificationResp = await fetch('http://localhost:8080/api/v1/webauthn/finish_registration', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          challenge_id: challenge_id,
          credential: credentialJSON
        })
      });

      if (!verificationResp.ok) {
        throw new Error(`HTTP error! status: ${verificationResp.status}`);
      }

      const verificationJSON = await verificationResp.json();

      if (verificationJSON && verificationJSON.verified) {
        setSuccessMessage('Success! Passkey registered successfully.');
      } else {
        setErrorMessage(`Registration failed: ${JSON.stringify(verificationJSON, null, 2)}`);
      }
    } catch (error: any) {
      console.error('Registration failed:', error)
      if (!errorMessage) { // Only set if not already set by inner catch
        setErrorMessage(`Registration failed: ${error.message}`)
      }
    } finally {
      setIsRegistering(false)
    }
  }


  // const handleAuthentication = async () => {
  //   // Reset success/error messages
  //   setSuccessMessage('')
  //   setErrorMessage('')
  //   setIsAuthenticating(true)

  //   try {
  //     // GET authentication options from the endpoint that calls
  //     // @simplewebauthn/server -> generateAuthenticationOptions()
  //     const resp = await fetch('http://localhost:5173/generate-authentication-options')

  //     if (!resp.ok) {
  //       throw new Error(`HTTP error! status: ${resp.status}`)
  //     }

  //     const optionsJSON = await resp.json()

  //     let asseResp: AuthenticationResponseJSON
  //     try {
  //       // Pass the options to the authenticator and wait for a response
  //       asseResp = await startAuthentication({ optionsJSON })
  //     } catch (error: any) {
  //       // Some basic error handling
  //       setErrorMessage(error.message || 'An error occurred during authentication')
  //       throw error
  //     }

  //     // POST the response to the endpoint that calls
  //     // @simplewebauthn/server -> verifyAuthenticationResponse()
  //     const verificationResp = await fetch('http://localhost:5173/verify-authentication', {
  //       method: 'POST',
  //       headers: {
  //         'Content-Type': 'application/json',
  //       },
  //       body: JSON.stringify(asseResp),
  //     })

  //     if (!verificationResp.ok) {
  //       throw new Error(`HTTP error! status: ${verificationResp.status}`)
  //     }

  //     // Wait for the results of verification
  //     const verificationJSON = await verificationResp.json()

  //     // Show UI appropriate for the `verified` status
  //     if (verificationJSON && verificationJSON.verified) {
  //       setSuccessMessage('Success! You are authenticated.')
  //     } else {
  //       setErrorMessage(`Oh no, something went wrong! Response: ${JSON.stringify(verificationJSON, null, 2)}`)
  //     }
  //   } catch (error: any) {
  //     console.error('Authentication failed:', error)
  //     if (!errorMessage) { // Only set if not already set by inner catch
  //       setErrorMessage(`Authentication failed: ${error.message}`)
  //     }
  //   } finally {
  //     setIsAuthenticating(false)
  //   }
  // }

  function base64UrlToBuffer(base64url: string): ArrayBuffer {
    const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
    const padded = base64.padEnd(base64.length + (4 - base64.length % 4) % 4, '=');
    const binary = atob(padded);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
  }
  
  function bufferToBase64Url(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    const base64 = btoa(binary);
    return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
  }


  return (
    <div className="App">
      <header className="App-header">
        <FlowLogo />
        <h1>Flow Web</h1>
        <p>Welcome to Flow - A modern web application</p>

        <div className="card">
          <h2>Passkey Actions</h2>
          <div style={{ display: 'flex', gap: '1rem', justifyContent: 'center' }}>
            <button
              onClick={handleRegistration}
              disabled={isRegistering || isAuthenticating}
              style={{
                backgroundColor: (isRegistering || isAuthenticating) ? '#ccc' : '#646cff',
                cursor: (isRegistering || isAuthenticating) ? 'not-allowed' : 'pointer'
              }}
            >
              {isRegistering ? 'Registering...' : 'Register Passkey'}
            </button>

            {/* <button
              onClick={handleAuthentication}
              disabled={isAuthenticating || isRegistering}
              style={{
                backgroundColor: (isAuthenticating || isRegistering) ? '#ccc' : '#646cff',
                cursor: (isAuthenticating || isRegistering) ? 'not-allowed' : 'pointer'
              }}
            >
              {isAuthenticating ? 'Authenticating...' : 'Authenticate'}
            </button> */}
          </div>

          {successMessage && (
            <div style={{ color: 'green', marginTop: '1rem' }}>
              {successMessage}
            </div>
          )}

          {errorMessage && (
            <div style={{ color: 'red', marginTop: '1rem', whiteSpace: 'pre-wrap' }}>
              {errorMessage}
            </div>
          )}
        </div>

        <ElectronInfo />

        <div className="card">
          <button onClick={() => setCount((count) => count + 1)}>
            count is {count}
          </button>
          <p>
            Edit <code>src/App.jsx</code> and save to test HMR
          </p>
        </div>

        <p className="read-the-docs">Click on the Flow logo to learn more</p>
      </header>
    </div>
  );
}

export default App;
