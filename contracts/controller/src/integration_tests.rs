#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Empty};
    use cw_multi_test::{Contract, ContractWrapper, Executor};
    use messages::contract::{
        execute as messagesExecute, instantiate as messagesInstantiate, query as messagesQuery,
    };
    use profiles::contract::{
        execute as profilesExecute, instantiate as profilesInstantiate, query as profilesQuery,
    };

    use crate::{
        contract::{execute, instantiate, query, reply},
        msg::{
            ContractAddressesResponse, ExecuteMsg as ControllerExecuteMsg,
            InstantiateMsg as ControllerInstantiateMsg, QueryMsg as ControllerQueryMsg,
        },
        state::Config,
    };
    use utils::query::{ProfileInfo, ProfilesQueryMsg};

    use cosmwasm_std::{coins, Addr};
    use cw_multi_test::AppBuilder;
    const DENOM: &str = "udevcore";

    fn controller_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    fn profiles_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(profilesExecute, profilesInstantiate, profilesQuery);
        Box::new(contract)
    }

    fn messages_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(messagesExecute, messagesInstantiate, messagesQuery);
        Box::new(contract)
    }

    #[test]
    fn test_contract_flow() {
        let admin = Addr::unchecked("admin");
        let mut app = AppBuilder::new().build(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &admin, coins(10000, DENOM))
                .unwrap();
        });

        let code_id_controller = app.store_code(controller_contract());
        let code_id_profiles = app.store_code(profiles_contract());
        let code_id_messages = app.store_code(messages_contract());

        let contract_addr = app
            .instantiate_contract(
                code_id_controller,
                admin.clone(),
                &ControllerInstantiateMsg {
                    code_id_profiles,
                    code_id_messages,
                    message_max_len: 5000,
                    message_query_default_limit: 50,
                    message_query_max_limit: 500,
                    create_profile_cost: Some(coin(100, DENOM)),
                    send_message_cost: Some(coin(10, DENOM)),
                },
                &[],
                "Controller",
                None,
            )
            .unwrap();

        let resp: Config = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &ControllerQueryMsg::Config {})
            .unwrap();

        assert_eq!(
            resp,
            Config {
                message_max_len: 5000,
                message_cost: Some(coin(10, DENOM)),
                profile_cost: Some(coin(100, DENOM))
            }
        );

        let resp: ContractAddressesResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &ControllerQueryMsg::ContractAddresses {},
            )
            .unwrap();

        let msg_create_profile = &ControllerExecuteMsg::CreateProfile {
            user_id: "myuser".to_owned(),
            pubkey: "mypubkey".to_owned(),
        };

        let send_funds = coins(100, DENOM);

        app.execute_contract(
            admin.clone(),
            contract_addr.clone(),
            msg_create_profile,
            &send_funds,
        )
        .unwrap();

        let resp_profiles: ProfileInfo = app
            .wrap()
            .query_wasm_smart(
                resp.profiles_contract_addr,
                &ProfilesQueryMsg::AddressInfo { address: admin },
            )
            .unwrap();

        assert_eq!(resp_profiles.user_id, "myuser");
    }
}
