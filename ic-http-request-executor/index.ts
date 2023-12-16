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

let ws = openWsConnection();

function openWsConnection() {
  const _ws = new IcWebSocket(gatewayUrl, {}, wsConfig);

  _ws.onopen = () => {
    console.log("WebSocket connected with principal", _ws["_clientKey"].client_principal.toString());
  };

  _ws.onmessage = async (ev) => {
    const incomingMessage = ev.data;
    // console.log("Message", incomingMessage);

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
        "\nExecuting HTTP request:",
        "\nurl:", url.toString(),
        "\nmethod:", method,
        "\nheaders:", headers,
        "\nbody bytes:", body?.length,
        "\nbody:", body ? new TextDecoder().decode(body) : null
      );

      try {
        const response = await fetch(url, {
          method,
          headers,
          body,
        });

        const responseBody = new Uint8Array(await response.arrayBuffer());

        console.log(
          "HTTP response:",
          "\nurl:", request.url,
          "\nstatus:", response.status,
          "\nbody bytes:", responseBody.byteLength,
          // "\nbody:", new TextDecoder().decode(responseBody),
        );

        _ws.send({
          HttpResponse: [
            requestId,
            {
              status: BigInt(response.status),
              headers: Array.from(response.headers.entries()).map(([key, value]) => ({
                name: key,
                value,
              })),
              body: responseBody,
            },
          ],
        });

        console.log("Sent response over WebSocket.");
      } catch (e) {
        console.error("http-over-ws: error", e);
        _ws.send({
          Error: [[requestId], String(e)],
        });
      }
    } else if ("Error" in incomingMessage) {
      console.error("http-over-ws: incoming error:", incomingMessage.Error);
    }
  };

  _ws.onclose = (ev) => {
    console.warn("WebSocket disconnected. Reason:", ev.reason);
    console.log("Reconnecting...");
    ws = openWsConnection();
  };

  _ws.onerror = (ev) => {
    console.error("WebSocket error:", ev.message);
  };
};
