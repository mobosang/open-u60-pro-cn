import type { Metadata } from "next";
import { Callout } from "@/components/ui/Callout";
import { CodeBlock } from "@/components/ui/CodeBlock";

export const metadata: Metadata = {
  title: "故障排查",
  description: "zte-agent常见问题及解决方案。",
};

export default function TroubleshootingPage() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="mb-2 font-display text-2xl font-bold">
          故障排除
        </h1>
        <p className="text-text-dim">
          常见问题及其解决方法。
        </p>
      </div>

      <section>
        <h2 className="mb-3 text-lg font-semibold">
          登录失败/密码错误
        </h2>
        <p className="mb-3 text-sm text-text-dim">
          The &quot;Router Password&quot; field expects the admin password for your
          router&apos;s web interface (typically accessed at 192.168.0.1). This
          is not the WiFi password.
        </p>
        <Callout type="info">
          If you&apos;ve never changed it, try the default password printed on the
          router&apos;s label.
        </Callout>
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">
          连接超时
        </h2>
        <p className="mb-3 text-sm text-text-dim">
          Ensure you are connected to the router&apos;s WiFi or Ethernet.
          The agent communicates directly with the router at the gateway
          IP (default: 192.168.0.1).
        </p>
        <p className="text-sm text-text-dim">
          If you&apos;ve changed the router&apos;s LAN IP, update the
          IP address in the companion app or API calls accordingly.
        </p>
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">
          部署后代理没有响应
        </h2>
        <p className="mb-3 text-sm text-text-dim">
          If the agent isn&apos;t reachable at port 9090:
        </p>
        <ul className="list-inside list-disc space-y-1 text-sm text-text-dim">
          <li>等待 5-10 秒，然后重试</li>
          <li>
            检查代理是否正在通过 SSH 运行：
          </li>
        </ul>
        <CodeBlock
          code={`ssh -p 2222 root@192.168.0.1 'ps | grep zte-agent'`}
          language="bash"
          className="mt-2"
        />
        <p className="mt-2 text-sm text-text-dim">
          If the process isn&apos;t running, check the boot script:
        </p>
        <CodeBlock
          code={`ssh -p 2222 root@192.168.0.1 'cat /data/local/tmp/start_zte_agent.sh'
ssh -p 2222 root@192.168.0.1 'sh /data/local/tmp/start_zte_agent.sh'`}
          language="bash"
          className="mt-2"
        />
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">
          CORS 错误
        </h2>
        <p className="text-sm text-text-dim">
          The zte-agent sends no CORS headers, and it refuses requests from
          other websites that change settings, so a page on another site
          cannot use the API from your browser. Use the native apps, or the
          agent&apos;s own web UI at http://192.168.0.1:9090/.
        </p>
      </section>
    </div>
  );
}
