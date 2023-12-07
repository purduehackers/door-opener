import { FC, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Badge } from '../components/Badge/Badge';
import { listen } from '@tauri-apps/api/event';

export type AuthUpdatePayload = {
    authState: number;
};

// this code is for debugging, DO NOT REMOVE
// let current_auth_state = 0;
// let should_accept = true;
// let next_iter_resets = false;

const App: FC = () => {
    const [queuedAuthState, setQueuedAuthState] = useState(-1);
    const [animatingAuthState, setAnimatingAuthState] = useState(-1);

    const [authState, setAuthState] = useState(0);
    const [showWelcome, setShowWelcome] = useState(true);

    const [activeMessage, setActiveMessage] = useState(0);

    const [activeMessageTimeout, setActiveMessageTimeout] = useState(0);
    const [activeMessageTimeoutRunning, setActiveMessageTimeoutRunning] =
        useState(false);

    const updateAuthState = (authState: AuthUpdatePayload) => {
        setQueuedAuthState(authState.authState);

        console.log(authState.authState);
    };

    // this code is for debugging, DO NOT REMOVE
    // useEffect(() => {
    //     let unlisten = setInterval(() => {
    //         updateAuthState({ authState: current_auth_state });

    //         current_auth_state += 1;

    //         if (next_iter_resets) {
    //             next_iter_resets = false;
    //             current_auth_state = 0;
    //         } else if (current_auth_state == 2) {
    //             next_iter_resets = true;

    //             if (!should_accept) {
    //                 current_auth_state += 1;
    //             }

    //             should_accept = !should_accept;
    //         }
    //     }, 5000);

    //     return () => {
    //         clearInterval(unlisten);
    //     };
    // });

    const updateActiveMessage = (message: number, after: number) => {
        if (activeMessageTimeoutRunning) {
            clearTimeout(activeMessageTimeout);
            setActiveMessageTimeoutRunning(false);
        }

        if (after == 0) {
            setActiveMessage(message);
        } else {
            setActiveMessageTimeout(
                setTimeout(() => setActiveMessage(message), after)
            );
            setActiveMessageTimeoutRunning(true);
        }
    };

    useEffect(() => {
        if (animatingAuthState == -1) return;

        invoke('set_led_effect', { number: animatingAuthState });

        switch (animatingAuthState) {
            case 0:
                setAuthState(0);
                setShowWelcome(true);

                setTimeout(() => {
                    setAnimatingAuthState(-1);
                }, 1000);

                break;
            case 1:
                setShowWelcome(false);

                updateActiveMessage(0, 0);

                setTimeout(() => {
                    setAuthState(1);

                    setTimeout(() => {
                        setAnimatingAuthState(-1);
                    }, 1000);
                }, 500);

                break;
            case 2:
                setAuthState(2);

                updateActiveMessage(1, 0);

                setTimeout(() => {
                    setShowWelcome(true);

                    setTimeout(() => {
                        setAnimatingAuthState(-1);
                    }, 500);

                    updateActiveMessage(0, 5000);
                }, 1500);

                break;
            case 3:
                setAuthState(3);

                updateActiveMessage(2, 0);

                setTimeout(() => {
                    setShowWelcome(true);

                    setTimeout(() => {
                        setAnimatingAuthState(-1);
                    }, 500);

                    updateActiveMessage(0, 10000);
                }, 1500);

                break;
        }
    }, [animatingAuthState]);

    useEffect(() => {
        if (animatingAuthState < 0 && queuedAuthState >= 0) {
            setAnimatingAuthState(queuedAuthState);
            setQueuedAuthState(-1);
        }
    }, [animatingAuthState, queuedAuthState]);

    useEffect(() => {
        const unlisten = listen('auth-update', (e) => {
            updateAuthState(e.payload as AuthUpdatePayload);
        });

        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    return (
        <div className="bg-neutral-950 h-screen w-screen overflow-hidden">
            <div className='animate-[background-scroll_30s_linear_infinite] bg-[length:100px] bg-[url("ph-logo-tilable.png")] min-h-screen min-w-screen'></div>

            <div
                className={`transition-all duration-500 absolute w-screen -translate-x-1/2 -translate-y-1/2 top-1/2 left-1/2 bg-neutral-950 border-y-ph-yellow border-x-transparent border-dashed border-4 ${
                    showWelcome && activeMessage == 0
                        ? 'opacity-1'
                        : 'opacity-0'
                }`}
            >
                <h1 className="text-8xl text-ph-yellow m-8">
                    Welcome to Hack Night
                </h1>
                <h1 className="text-5xl text-ph-yellow m-8">
                    Scan your passport to<br/>start
                </h1>
            <img
                className={`transition-all duration-500 h-[6rem] absolute text-ph-yellow bottom-8 right-8 bg-neutral-950 ${
                    showWelcome && activeMessage == 0
                        ? 'opacity-1'
                        : 'opacity-0'
                }`}
                src="./doorbell-qr.png"
            ></img>
            <img
                className={`transition-all duration-500 w-[10rem] absolute text-ph-yellow -translate-y-[6rem] bottom-8 right-4 bg-neutral-950 ${
                    showWelcome && activeMessage == 0
                        ? 'opacity-1'
                        : 'opacity-0'
                }`}
                src="./qr-pointer.svg"
            ></img>
            </div>

            <div
                className={`transition-all duration-500 absolute w-screen -translate-x-1/2 -translate-y-1/2 top-1/2 left-1/2 bg-neutral-950 border-y-ph-yellow border-x-transparent border-dashed border-4 ${
                    showWelcome && activeMessage == 1
                        ? 'opacity-1'
                        : 'opacity-0'
                }`}
            >
                <h1 className="text-8xl text-ph-yellow m-8">Welcome back!</h1>
                <h1 className="text-5xl text-ph-yellow m-8">
                    Please be mindful of the door opening
                </h1>
            </div>

            <div
                className={`transition-all duration-500 absolute w-screen -translate-x-1/2 -translate-y-1/2 top-1/2 left-1/2 bg-neutral-950 border-y-ph-yellow border-x-transparent border-dashed border-4 ${
                    showWelcome && activeMessage == 2
                        ? 'opacity-1'
                        : 'opacity-0'
                }`}
            >
                <h1 className="text-8xl text-ph-yellow m-8">Invalid Passport!</h1>
                <h1 className="text-5xl text-ph-yellow m-8">
                    Please try again or scan the QR code to ring the doorbell manually!
                </h1>
                <img
                    className="h-[12rem] absolute text-ph-yellow top-8 right-8"
                    src="./doorbell-qr.png"
                ></img>
            </div>

            <Badge state={authState} />
        </div>
    );
};

export default App;
