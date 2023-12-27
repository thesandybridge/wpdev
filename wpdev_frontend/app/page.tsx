'use client'

import styles from './page.module.css'
import Instance from './components/Instance'
import {useEffect, useState, useRef, useMemo} from 'react'
import { w3cwebsocket as WebSocket } from 'websocket'
import {InstanceStatus} from './types/globalTypes'

import {
    faHome,
    faGear
} from '@fortawesome/free-solid-svg-icons'
import FaIcon from './components/FaIcon'

export default function Home() {
    const [instances, setInstances] = useState<Instance[]>([])
    const api = 'http://127.0.0.1:8000/api/'
    const wsUrl = 'ws://127.0.0.1:8000/api/instances/ws'
    const ws = useRef<WebSocket | null>(null)
    const [isLoading, setIsLoading] = useState(false);

    const getStatusOrder = (status: InstanceStatus) => {
        const order = [
            InstanceStatus.Running,
            InstanceStatus.PartiallyRunning,
            InstanceStatus.Restarting,
            InstanceStatus.Stopped,
            InstanceStatus.Exited,
            InstanceStatus.Dead,
            InstanceStatus.Unknown
        ];
        return order.indexOf(status);
    };

    const sortedInstances = useMemo(() => {
        return [...instances].sort((a, b) => {
            return getStatusOrder(a.status) - getStatusOrder(b.status);
        });
    }, [instances]);

    const handleButtonClick = async (action: string) => {
        setIsLoading(true);
        try {
            const response = await fetch(`${api}instances/${action}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            });


            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            requestInspect();

        } catch (error) {
            console.error(error);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        ws.current = new WebSocket(wsUrl);
        console.log('Attempting to connect to WebSocket');

        ws.current.onopen = () => {
            console.log('WebSocket connected');
            requestInspect();
        };

        ws.current.onmessage = (event: any) => {
            try {
                const data = JSON.parse(event.data);
                console.log('Received data:', data);
                setInstances(data);
            } catch (error) {
                console.error('Error parsing WebSocket message:', error);
            }
        };

        ws.current.onerror = (event: any) => {
            console.error('WebSocket error:', event);
        };

        ws.current.onclose = (event: any) => {
            console.log(`WebSocket connection closed: ${event.code} - ${event.reason}`);
            // Implement reconnection logic if needed
        };

        return () => {
            if (ws.current) {
                ws.current.close();
            }
        };
    }, []);

    const requestInspect = () => {
        if (ws.current && ws.current.readyState === WebSocket.OPEN) {
            ws.current.send('request_inspect');
        }
    };

    return (
        <div className={styles.grid}>
            <aside className={styles.sidebar}>
                <nav>
                    <ul>
                        <li><FaIcon icon={faHome} /></li>
                        <li><FaIcon icon={faGear} /></li>
                    </ul>
                </nav>
            </aside>
            <main className={styles.main}>
                <header className="">
                    <h1>Instances</h1>
                    <nav role="menu" className={styles.controls}>
                        <button onClick={() => handleButtonClick('create')}>Create Instance</button>
                        <button onClick={() => handleButtonClick('start_all')}>Start All</button>
                        <button onClick={() => handleButtonClick('stop_all')}>Stop All</button>
                        <button onClick={() => handleButtonClick('restart_all')}>Restart All</button>
                        <button onClick={() => handleButtonClick('purge')}>Purge All</button>
                    </nav>
                </header>
                <div className="instances">
                    {sortedInstances && sortedInstances.length > 0 ? sortedInstances.map((instance, i) => (
                        <Instance
                            key={i}
                            data={instance}
                            api={api}
                            fetchInstances={requestInspect}
                            isAllLoading={isLoading}
                        />
                    )) : <p>No instances found</p>}
                </div>
            </main>
        </div>
    )
}
