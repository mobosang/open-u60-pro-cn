import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "法律 — OpenU60",
  description:
    "OpenU60 项目的法律免责声明、保修信息和商标声明。",
};

export default function LegalPage() {
  return (
    <div className="mx-auto max-w-3xl px-6 py-16">
      <h1 className="mb-2 font-display text-2xl font-bold">
        法律免责声明
      </h1>
      <p className="mb-8 text-sm text-text-dim">最后更新：2026 年 3 月 10 日</p>

      <div className="space-y-6 text-sm leading-relaxed text-text-dim">
        <section>
          <h2 className="mb-2 text-base font-semibold text-text">免责声明</h2>
          <p>
            OpenU60是一个独立的开源项目。它不隶属
            与 ZTE Corporation 合作、认可或赞助。 ZTE、U60 Pro、
            和 MU5250 是 ZTE Corporation 的商标。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">许可证</h2>
          <p className="mb-2">
            OpenU60 is released under the{" "}
            <a
              href="https://github.com/jesther-ai/open-u60-pro/blob/main/LICENSE"
              className="text-accent hover:underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              MIT 许可证
            </a>
            .
          </p>
          <p className="rounded border border-border bg-surface-1 px-3 py-2 font-mono text-xs leading-relaxed">
            THE SOFTWARE IS PROVIDED &quot;AS IS&quot;, WITHOUT WARRANTY OF ANY
            KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
            OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
            NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
            BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
            ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
            CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
            SOFTWARE.
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            无保证
          </h2>
          <p>
            This software is provided &quot;as is&quot; without warranty of any
            kind, either express or implied. There is no guarantee that the
            software will be error-free, uninterrupted, or fit for any
            particular purpose. You use it entirely at your own risk.
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            责任限制
          </h2>
          <p>
            本项目的作者和贡献者不承担任何责任
            任何设备损坏、制造商保修失效、变砖
            路由器、数据丢失、网络中断或任何其他后果
            因使用本软件而产生的。您承担所有责任
            对您的设备所做的任何更改。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            预期用途
          </h2>
          <p>
            该软件适用于您个人拥有的设备，并且
            拥有完全的管理访问权限。它不适合
            未经授权访问不属于您或您控制的设备。你是
            全权负责确保您的使用符合适用的规定
            法律法规。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            设备修改警告
          </h2>
          <p>
            设备上的代理和配套应用程序公开控制端点，
            可以修改固件设置、网络配置并执行
            潜在的破坏性操作，包括恢复出厂设置和重新启动。
            这些行为可能会使您的制造商保修失效。通过使用这个
            软件，您对所做的任何更改承担全部责任
            您的设备。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            逆向工程
          </h2>
          <p>
            该项目中的所有逆向工程仅用于
            适用合理使用下的互操作性和教育目的
            和互操作性例外（包括但不限于欧盟
            指令 2009/24/EC 和美国合理使用原则）。无专有
            该项目包含 ZTE 公司的源代码。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            无专有代码
          </h2>
          <p>
            没有 ZTE 专有源代码、二进制文件或受版权保护的资产
            作为该项目的一部分分发。加密密钥和协议
            详细信息来自公开的社区研究和
            标准互操作性分析。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            商标
          </h2>
          <p>
            ZTE、U60 Pro 和 MU5250 是 ZTE Corporation 的商标。
            Qualcomm 和 Snapdragon 是高通公司的商标。全部
            其他商标是其各自所有者的财产。这些
            本项目中使用的名称仅用于识别目的，并且
            并不意味着认可。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">联系方式</h2>
          <p>
            For legal inquiries, please open an issue on the{" "}
            <a
              href="https://github.com/jesther-ai/open-u60-pro/issues"
              className="text-accent hover:underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub 存储库
            </a>
            .
          </p>
        </section>

        <section className="border-t border-border pt-6">
          <p className="text-xs">
            OpenU60 不隶属于 ZTE、不由其认可或赞助
            公司。 ZTE 和 U60 Pro 是 ZTE Corporation 的商标。
          </p>
        </section>
      </div>
    </div>
  );
}
