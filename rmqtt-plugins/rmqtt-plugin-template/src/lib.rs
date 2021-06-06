use async_trait::async_trait;

use rmqtt::{
    broker::hook::{Handler, HookResult, Parameter, Register, ReturnType, Type},
    plugin::Plugin,
    Result, Runtime,
};

#[inline]
pub async fn init<N: Into<String>, D: Into<String>>(
    runtime: &'static Runtime,
    name: N,
    descr: D,
    default_startup: bool,
) -> Result<()> {
    runtime
        .plugins
        .register(Box::new(Template::new(runtime, name.into(), descr.into()).await), default_startup)
        .await?;
    Ok(())
}

struct Template {
    _runtime: &'static Runtime,
    name: String,
    descr: String,
    register: Box<dyn Register>,
}

impl Template {
    #[inline]
    async fn new(runtime: &'static Runtime, name: String, descr: String) -> Self {
        let register = runtime.extends.hook_mgr().await.register();
        Self { _runtime: runtime, name, descr, register }
    }
}

#[async_trait]
impl Plugin for Template {
    #[inline]
    async fn init(&mut self) -> Result<()> {
        log::debug!("{} init", self.name);
        self.register.add(Type::ClientConnack, Box::new(HookHandler::new())).await;
        self.register.add(Type::ClientSubscribe, Box::new(HookHandler::new())).await;
        self.register.add(Type::ClientUnsubscribe, Box::new(HookHandler::new())).await;
        self.register.add(Type::MessageDelivered, Box::new(HookHandler::new())).await;
        self.register.add(Type::MessagePublish, Box::new(HookHandler::new())).await;
        self.register.add_priority(Type::ClientSubscribeCheckAcl, 10, Box::new(HookHandler::new())).await;
        self.register.add_priority(Type::GrpcMessageReceived, 10, Box::new(HookHandler::new())).await;

        Ok(())
    }

    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    async fn start(&mut self) -> Result<()> {
        log::info!("{} start", self.name);
        self.register.start().await;
        Ok(())
    }

    #[inline]
    async fn stop(&mut self) -> Result<bool> {
        log::info!("{} stop", self.name);
        self.register.stop().await;
        Ok(true)
    }

    #[inline]
    fn version(&self) -> &str {
        "0.1.1"
    }

    #[inline]
    fn descr(&self) -> &str {
        &self.descr
    }
}

struct HookHandler {}

impl HookHandler {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Handler for HookHandler {
    async fn hook(&self, param: &Parameter, acc: Option<HookResult>) -> ReturnType {
        match param {
            Parameter::ClientConnack(connect_info, r) => {
                log::debug!("client connack, {:?}, {:?}", connect_info, r);
            }
            Parameter::ClientSubscribe(_session, c, subscribe) => {
                log::debug!("{:?} client subscribe, {:?}", c.id, subscribe);
                //let mut topic_filter = subscribe.topic_filter.clone();
                //topic_filter.insert(0, Level::Normal("PPP".into()));
                //return (true, Some(HookResult::TopicFilter(Some(topic_filter))))
            }
            Parameter::ClientUnsubscribe(_session, c, unsubscribe) => {
                log::debug!("{:?} client unsubscribe, {:?}", c.id, unsubscribe);
                //let mut topic_filter = (*unsubscribe).clone();
                //topic_filter.insert(0, Level::Normal("PPP".into()));
                //return (true, Some(HookResult::TopicFilter(Some(topic_filter))))
            }
            Parameter::MessagePublish(_session, c, publish) => {
                log::debug!("{:?} message publish, {:?}", c.id, publish);
            }
            Parameter::MessageDelivered(_session, c, from, _publish) => {
                log::debug!("{:?} MessageDelivered, {:?}", c.id, from);
            }
            Parameter::ClientSubscribeCheckAcl(_s, _c, subscribe) => {
                log::debug!("{:?} ClientSubscribeCheckAcl, {:?}", _c.id, subscribe);
            }
            _ => {
                log::error!("unimplemented, {:?}", param)
            }
        }
        (true, acc)
    }
}
