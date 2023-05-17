import * as kx from "@pulumi/kubernetesx"
import * as pulumi from "@pulumi/pulumi"

export default function cloudflareTunnel(
    namespace: pulumi.Output<string>,
    url: string) {
    const cloudflaredPod = new kx.PodBuilder({
        containers: [{
            name: "cloudflare-tunnel",
            image: "cloudflare/cloudflared:latest",
            command: ["cloudflared", "tunnel", "--url", url, '--protocol', 'http2'],
        }]
    })

    let deployName = pulumi.interpolate `${namespace}-cloudflare-tunnel`
    
    new kx.Deployment("cloudflare-tunnel", {
        metadata: {
            name: deployName,
            namespace: namespace
        },
        spec: cloudflaredPod.asDeploymentSpec({ replicas: 1 })
    })
}