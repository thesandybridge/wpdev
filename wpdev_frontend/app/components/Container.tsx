import {Container} from '../types/globalTypes';
import Icon from '../icons/Icon';

const status_color = {
    'Running': '#87e084',
    'Restarting': '#c18145',
    'Stopped': '#ececec',
    'Exited': '#e08484',
    'Dead': '#a5181b',
    'Unknown': '#c784e0',
    'Paused': '#ececec',
    'NotFound': '#18a574',
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
