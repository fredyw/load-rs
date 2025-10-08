#!/bin/bash

set -ueo pipefail

#==============================================================================
# Generate CA
#==============================================================================
# Generate CA private key.
openssl genpkey -algorithm RSA -out ca.key
# Generate CA certificate (self-signed).
openssl req -new -x509 -key ca.key -out ca.crt -subj "/CN=Test CA" -extensions v3_ca -days 3650

#==============================================================================
# Generate server certificate
#==============================================================================
# Generate server private key.
openssl genpkey -algorithm RSA -out server.key
# Create a config file for the server SAN.
echo "subjectAltName = DNS:localhost,IP:127.0.0.1,IP:::1" > server.ext
# Generate a certificate signing request (CSR) for the server.
openssl req -new -key server.key -out server.csr -subj "/CN=localhost"
# Sign the server certificate with your CA.
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -extfile server.ext -days 3650

#==============================================================================
# Generate client certificate
#==============================================================================
# Generate client private key.
openssl genpkey -algorithm RSA -out client.key
# Create a config file for the client certificate extensions.
echo "extendedKeyUsage = clientAuth" > client.ext
# Generate a CSR for the client.
openssl req -new -key client.key -out client.csr -subj "/CN=Test Client"
# Sign the client certificate with your CA.
openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out client.crt -extfile client.ext -days 3650

#==============================================================================
# Generate untrusted CA for testing
#==============================================================================
echo "Generating untrusted client certificate..."
openssl genpkey -algorithm RSA -out untrusted-ca.key
openssl req -new -x509 -key untrusted-ca.key -out untrusted-ca.crt -subj "/CN=Untrusted Test CA" -extensions v3_ca -days 3650

#==============================================================================
# Generate untrusted client certificate for testing
#==============================================================================
openssl genpkey -algorithm RSA -out untrusted-client.key
openssl req -new -key untrusted-client.key -out untrusted-client.csr -subj "/CN=Untrusted Test Client"
openssl x509 -req -in untrusted-client.csr -CA untrusted-ca.crt -CAkey untrusted-ca.key -CAcreateserial -out untrusted-client.crt -days 3650

#==============================================================================
# Cleanup temporary files
#==============================================================================
rm -rf ./*.csr ./*.srl ./*.ext

