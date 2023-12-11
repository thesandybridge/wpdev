import {Instance, InstanceStatus} from '../types/globalTypes'

interface ButtonProps {
    data: Instance
    handleButtonClick: (action: string) => void
        isButtonLoading: (action: string) => boolean
    globalLoading?: boolean
}

const activeStatuses = new Set([InstanceStatus.Running, InstanceStatus.Restarting, InstanceStatus.PartiallyRunning])
const inactiveStatuses = new Set([InstanceStatus.Stopped, InstanceStatus.Exited, InstanceStatus.Dead, InstanceStatus.Unknown])

export default function ControlButtons({
    data,
    handleButtonClick,
    isButtonLoading,
    globalLoading
}: ButtonProps) {
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
