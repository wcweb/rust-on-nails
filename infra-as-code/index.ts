import * as k8s from "@pulumi/kubernetes"
import * as kx from "@pulumi/kubernetesx"
import setupCluster from './cluster-setup'
import setupDatabase from './database'
import cloudflareTunnel from './cloudflare'
// Add a postgres operator and anything else apllications need
const cloudnativePg = setupCluster()

// Setup a namespace for our application
const applicationNameSpace = new k8s.core.v1.Namespace('rust-on-nails', {
    metadata: {
        name: 'rust-on-nails'
    },
})

setupDatabase(applicationNameSpace, cloudnativePg)

const applicationPods = new kx.PodBuilder({
    containers: [{
        name: "application",
        image: `ghcr.io/purton-tech/nails-example:latest`,
        imagePullPolicy: 'IfNotPresent',
        ports: { http: 3000 },
        env: [
            {
                name: 'APP_DATABASE_URL', valueFrom: {
                    secretKeyRef: {
                        name: 'database-urls',
                        key: 'application-url'
                    }
                }
            },
        ]
    }],
    initContainers: [{
        // This runs the migrations when the pod starts.
        name: "application-migrations",
        image: `ghcr.io/purton-tech/nails-example-migrations:latest`,
        imagePullPolicy: 'IfNotPresent',
        env: [
            {
                name: 'DATABASE_URL', valueFrom: {
                    secretKeyRef: {
                        name: 'database-urls',
                        key: 'migrations-url'
                    }
                }
            },
        ]
    }]
})

const deployment =new kx.Deployment("application", {
    metadata: {
        name: "application",
        namespace: applicationNameSpace.metadata.name
    },
    spec: applicationPods.asDeploymentSpec({ replicas: 1 }) 
})

new k8s.core.v1.Service("application", {
    metadata: {
        name: "application",
        namespace: applicationNameSpace.metadata.name
    },
    spec: {
        ports: [
            { port: 3000, targetPort: 3000 }
        ],
        type: "ClusterIP",
        selector: {
            app: deployment.metadata.name
        }
    }
})

cloudflareTunnel(applicationNameSpace.metadata.name, "http://application:3000")