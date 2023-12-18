import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { config } from '@fortawesome/fontawesome-svg-core';
import '@fortawesome/fontawesome-svg-core/styles.css';
config.autoAddCss = false; // Prevents auto-adding the CSS

const FAIcon = ({ icon, ...props }) => <FontAwesomeIcon icon={icon} {...props} />;

export default FAIcon;

