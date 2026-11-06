export interface ComparisonRow {
  label: string;
  agent: string;
  official: string;
  agentGood: boolean;
}

export const comparisonData: ComparisonRow[] = [
  {
    label: "流程",
    agent: "1",
    official: "44",
    agentGood: true,
  },
  {
    label: "记忆 (RSS)",
    agent: "~0.8 MB",
    official: "225 MB",
    agentGood: true,
  },
  {
    label: "二进制大小",
    agent: "~2.3 MB",
    official: "~50+ MB combined",
    agentGood: true,
  },
  {
    label: "话题",
    agent: "~10",
    official: "~130+",
    agentGood: true,
  },
  {
    label: "遥测",
    agent: "无",
    official: "电话联系iot.zte.com.cn",
    agentGood: true,
  },
  {
    label: "应用程序",
    agent: "开源 iOS/Android",
    official: "闭源，仅限中文",
    agentGood: true,
  },
];
