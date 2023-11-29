FROM wordpress:latest

# Additional configurations can go here
# For example, installing specific PHP extensions, configuring .htaccess, etc.

# Set up WordPress with Nginx and PHP-FPM (if you prefer Nginx over Apache)
# FROM wordpress:php7.4-fpm
# RUN apt-get update && apt-get install -y nginx
# COPY nginx-site.conf /etc/nginx/sites-available/default

