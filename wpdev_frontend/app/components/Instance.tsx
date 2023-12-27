'use client'
import {useState} from 'react'

import ControlButtons from './ControlButtons'
import Container from './Container'

interface Props {
    data: Instance
    api: string
    isAllLoading: boolean
    fetchInstances: () => void
}

interface ButtonStatuses {
    [key: string]: boolean
}

interface LoadingState {
    [key: string]: boolean
}

export default function Instance(props: Props) {
    const { data, api, fetchInstances, isAllLoading} = props
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
            const response = await fetch(`${api}/instances/${data.uuid}/${action}`, {
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
        <div id={data.uuid} className={`instance${isInstanceLoading() || isAllLoading ? ' isLoading' : ''}`}>
            <header>
                <div className={`status_container ${data.status.toLowerCase()}`} title={data.status}/>
                <h4>Instance: {data.uuid}</h4>

            </header>
            <div className="instance_actions">
                <div className="site_links">
                    <a href={wordpress_path} target="_blank">
                        WordPress
                    </a>
                    <a href={adminer_path} target="_blank">
                        Adminer
                    </a>
                </div>
                <ControlButtons
                    data={data}
                    handleButtonClick={handleButtonClick}
                    isButtonLoading={isButtonLoading}
                    globalLoading={isInstanceLoading() || isDisabled}
                />
            </div>
            {data.containers && (
                <div className='instance_containers'>
                    {data.containers.map((container, i)=>
                    <Container
                        key={i}
                        container={container}
                    />
                    )}
                </div>
            )}
        </div>
    )
}
