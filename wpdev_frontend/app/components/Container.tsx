import {Container} from '../types/globalTypes';
import Icon from '../icons/Icon';

const status_color = {
    'Running': '#87e084',
    'Restarting': '#ececec',
    'Stopped': '#ffff00',
    'Exited': '#e08484',
    'Dead': '#e08484',
    'Unknown': '#c784e0',
    'Paused': '#ececec',
    'NotFound': '#e08484',
}

interface Props {
    container: Container
}

export default function Container(props: Props) {
    const { container } = props;
    return (
        <div className={`container ${container.container_image.toLowerCase()}`}>
            <div
            className={`status_container ${container.container_status.toString().toLowerCase()}`}
            title={container.container_status.toString()}
            />
            <div className="icon-wrapper">
                <Icon
                image={
                    container.container_image.toLowerCase()
                }
                fill={
                    status_color[container.container_status.toString()]
                }/>
            </div>
        </div>
    )
}
