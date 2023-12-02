import IcWebSocket, { createWsConfig, generateRandomIdentity } from "ic-websocket-js";
import { ic_side_services_backend, canisterId } from "./src/canister/declarations/ic_side_services_backend";

const icNetworkUrl = process.env.IC_NETWORK_URL as string;
const gatewayUrl = process.env.IC_WS_GATEWAY_URL as string;

const wsConfig = createWsConfig({
  canisterId,
  canisterActor: ic_side_services_backend,
  networkUrl: icNetworkUrl,
  identity: generateRandomIdentity(),
});

console.log("Canister ID:", canisterId);

const ws = new IcWebSocket(gatewayUrl, {}, wsConfig);

ws.onopen = () => {
  console.log("WebSocket connected");
};

ws.onmessage = async (ev) => {
  try {
    const incomingMessage = ev.data;
    console.log("Message", incomingMessage);

    if ("HttpRequest" in incomingMessage) {
      const requestId = incomingMessage.HttpRequest[0];
      const request = incomingMessage.HttpRequest[1];

      const url = new URL(request.url);
      const method = Object.keys(request.method)[0]; // workaround to get the candid enum
      const headers = new Headers(
        request.headers.map(({ name, value }) => [name, value] as [string, string])
      );
      const body = (request.body.length > 0 && method !== "GET")
        ? new Uint8Array(request.body[0]!)
        : null;

      console.log(
        "Sending HTTP request.",
        "\nurl:", url,
        "\nmethod:", method,
        "\nheaders:", headers,
        "\nbody bytes:", body?.length
      );

      const response = await fetch(url, {
        method,
        headers,
        body,
      });

      console.log("Got response from", request.url, "Status:", response.status);

      ws.send({
        HttpResponse: [
          requestId,
          {
            status: BigInt(response.status),
            headers: Array.from(response.headers.entries()).map(([key, value]) => ({
              name: key,
              value,
            })),
            body: new Uint8Array(await response.arrayBuffer()),
          },
        ],
      });

      console.log("Sent response");
    } else if ("Error" in incomingMessage) {
      console.error("http-over-ws: incoming error:", incomingMessage.Error);
    }
  } catch (e) {
    console.error("http-over-ws: error", e);
    ws.send({
      Error: String(e),
    });
  }
};

ws.onclose = (ev) => {
  console.log("WebSocket disconnected", ev);
};
