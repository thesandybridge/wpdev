'use client'
import {useState} from 'react'

import {Instance, InstanceStatus} from '../types/globalTypes'

interface Props {
    data: Instance
    api: string
    fetchInstances: () => void
}

const activeStatuses = new Set([InstanceStatus.Running, InstanceStatus.Restarting, InstanceStatus.PartiallyRunning])
const inactiveStatuses = new Set([InstanceStatus.Stopped, InstanceStatus.Exited, InstanceStatus.Dead, InstanceStatus.Unknown])

interface ButtonStatuses {
    [key: string]: boolean
}

interface LoadingState {
    [key: string]: boolean
}

interface ButtonProps {
    data: Instance
    handleButtonClick: (action: string) => void
    isButtonLoading: (action: string) => boolean
    globalLoading?: boolean
}

function ControlButtons({ data, handleButtonClick, isButtonLoading, globalLoading }: ButtonProps) {
    const isStartDisabled = () => activeStatuses.has(data.status) || isButtonLoading('start')
    const isStopDisabled = () => inactiveStatuses.has(data.status) || isButtonLoading('stop')
    const isRestartDisabled = () => inactiveStatuses.has(data.status) || isButtonLoading('restart')
    const isDeleteDisabled = () => isButtonLoading('delete')

    const buttonsConfig = [
        {
            action: 'start',
            label: 'Start',
            verb: 'Starting',
            disabled: isStartDisabled
        },
        {
            action: 'stop',
            label: 'Stop',
            verb: 'Stopping',
            disabled: isStopDisabled
        },
        {
            action: 'restart',
            label: 'Restart',
            verb: 'Restarting',
            disabled: isRestartDisabled
        },
        {
            action: 'delete',
            label: 'Delete',
            verb: 'Deleting',
            disabled: isDeleteDisabled
        }
    ]

    return (
        <div className='instance_actions'>
            {buttonsConfig.map(({ action, label, verb, disabled }) => (
                <button
                    key={action}
                    className='btn btn-primary'
                    onClick={() => handleButtonClick(action)}
                    disabled={disabled() || globalLoading}>
                    {isButtonLoading(action) ? `${verb}...` : label}
                </button>
            ))}
        </div>
    )
}

export default function Instance(props: Props) {
    const { data, api, fetchInstances} = props
    const wordpress_path = `${data.wordpress_data.site_url}`
    const adminer_path = `${data.wordpress_data.adminer_url}/?server=${data.uuid}-mysql&username=wordpress&db=wordpress`

    const [buttonStatuses, setButtonStatuses] = useState<ButtonStatuses>({})
    const [isLoading, setIsLoading] = useState<LoadingState>({})
    const [isDisabled, setIsDisabled] = useState<boolean>(false)

    const handleButtonClick = async (action: string) => {
        setButtonStatuses(prevStatuses => ({ ...prevStatuses, [`${data.uuid}_${action}`]: true }))
        setIsLoading(prevLoading => ({ ...prevLoading, [`${data.uuid}`]: true }))
        setIsDisabled(true)
        try {
            const response = await fetch(`${api}${data.uuid}/${action}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })


            if (!response.ok) {
                throw new Error(`Something went wrong when trying to process ${action}`)
            }

            fetchInstances()
        } catch (error) {
            console.error(error)
        } finally {
            // Set the status of the specific action to false
            setButtonStatuses(prevStatuses => ({ ...prevStatuses, [`${data.uuid}_${action}`]: false }))
            setIsLoading(prevLoading => ({ ...prevLoading, [`${data.uuid}`]: false }))
            setIsDisabled(false)
        }
    }

    const isButtonLoading = (action: string) => buttonStatuses[`${data.uuid}_${action}`]
    const isInstanceLoading = () => isLoading[data.uuid]

    return (
        <div id={data.uuid} className={`instance${isInstanceLoading() ? ' isLoading' : ''}`}>
            <header>
                <div className={`status_container ${data.status.toLowerCase()}`} title={data.status}/>
                <h4>Instance: {data.uuid}</h4>
            </header>
            <a href={wordpress_path} target="_blank">
                WordPress
            </a>
            <a href={adminer_path} target="_blank">
                Adminer
            </a>
            {data.container_statuses && (
                <div className='instance_containers'>
                    {Object.keys(data.container_statuses).map((uuid) => (
                        <div key={uuid}>
                            <div className={`status_container ${data.container_statuses[uuid].toLowerCase()}`} title={data.status}/>
                        </div>
                    ))}
                </div>
            )}
            <footer>
                <ControlButtons
                    data={data}
                    handleButtonClick={handleButtonClick}
                    isButtonLoading={isButtonLoading}
                    globalLoading={isInstanceLoading() || isDisabled}
                />
            </footer>
        </div>
    )
}
