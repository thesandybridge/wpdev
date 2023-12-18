import { Nginx, Adminer, WordPress, MySQL } from '../icons/icons'

export default function Icon(props) {
    const { image, fill } = props
    return (
        <>
            {image === 'nginx' && <Nginx fill={fill}/>}
            {image === 'adminer' && <Adminer fill={fill}/>}
            {image === 'wordpress' && <WordPress fill={fill}/>}
            {image === 'mysql' && <MySQL fill={fill}/>}
        </>

    )
}
