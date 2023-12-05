'use client'

import styles from './page.module.css'
import Instance from './components/Instance'
import {useEffect, useState, useRef, useMemo} from 'react'
import { w3cwebsocket as WebSocket } from 'websocket';
import {InstanceStatus} from './types/globalTypes'

export default function Home() {
    const [instances, setInstances] = useState<Instance[]>([])
    const api = 'http://127.0.0.1:8000/api/instances/'
    const wsUrl = 'ws://127.0.0.1:8000/api/instances/ws'
    const ws = useRef(null);

    const fetchData = () => {
        if (ws.current) {
            ws.current.send('request_inspect');
        }
    };

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

    const handleButtonClick = async (action: string, payload: any) => {
        try {
            const response = await fetch(`${api}${action}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            });


            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            fetchData();

        } catch (error) {
            console.error(error);
        }
    };

    useEffect(() => {
        ws.current = new WebSocket(wsUrl);

        ws.current.onopen = () => {
            fetchData();
        };

        ws.current.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                console.log(data);
                setInstances(data);
            } catch (error) {
                console.error('Error parsing WebSocket message:', error);
            }
        };

        ws.current.onerror = (event) => {
            console.error('WebSocket error:', event);
        };

        ws.current.onclose = (event) => {
            console.log(`WebSocket connection closed: ${event.code} - ${event.reason}`);
        };

        return () => {
            if (ws.current) {
                ws.current.close();
            }
        };
    }, []);

  return (
    <div className={styles.grid}>
        <aside className={styles.sidebar}>
            <nav>
                <ul>
                    <li>Home</li>
                    <li>Settings</li>
                </ul>
            </nav>
        </aside>
        <main className={styles.main}>
            <header>
                <h1>Instances</h1>
                <div className={styles.controls}>
                    <button onClick={() => handleButtonClick('create')}>Create Instance</button>
                    <button onClick={() => handleButtonClick('start_all')}>Start All</button>
                    <button onClick={() => handleButtonClick('stop_all')}>Stop All</button>
                    <button onClick={() => handleButtonClick('restart_all')}>Restart All</button>
                    <button onClick={() => handleButtonClick('purge')}>Purge All</button>
                </div>
            </header>
            <div className="instances">
                {sortedInstances && sortedInstances.map((instance, i) => (
                    <Instance
                        key={i}
                        data={instance}
                        api={api}
                        fetchInstances={fetchData}
                    />
                ))}
            </div>
        </main>
    </div>
  )
}
