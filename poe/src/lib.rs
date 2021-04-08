#![cfg_attr(not(feature = "std"), no_std)]


//引入依赖
use frame_support::{decl_module, decl_storage, decl_event, decl_error,ensure, dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//主程序
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

//存储单元
decl_storage! {
	trait Store for Module<T: Config> as TemplateModule {
		//先定义一个存储单元，用于存储存在归属信息
		//定义一个存储项 Proofs，给它一个default get散户，称之为 proofs
		//给Proofs设置类型为map，map的key是Vec，即存证hash值，由于无法得知使用哪些hash函数，所以使用变长类型u8
		//存证归属信息需要归属到一个人身上，以及它在哪个时间点被存储。这里定义一个tuple，给定两个参数，一个是用户信息（AccountId），一个是区块链时间（BlockNumber）
		//由于Vec是由用户输入，属于非安全的，这里得使用blake2_128_concat
		//一个简单存储单元编码完成，可以使用cargo check或者cargo build来检查语法是否有错误
		Proofs get(fn proofs): map hasher(blake2_128_concat) Vec<u8> => (T::AccountId,T::BlockNumber);
	
	} 
}

//事件
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		ClaimCreated(AccountId,Vec<u8>),  // 用户AccountId，存证内容 Vec<u8>
		ClaimRevoked(AccountId,Vec<u8>),
		move_claim(AccountId,Vec<u8>),
	}
);

//异常
decl_error! {
	pub enum Error for Module<T: Config> {
		ProofAlreadyExist,    // 存证已经存在
		ClaimNotExist,
		NotClaimOwner,
	}
}

//可调函数
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		// 创建存证，创建存证需要有两个关键参数：交易发送方origin，存证hash值claim，由于存证hash函数未知，也和decl_storage定义对应，这里使用变长Vec<u8>
        #[weight = 0]
		pub fn create_claim(origin,claim:Vec<u8>)->dispatch::DispatchResult{
			// 做必要检查，检查内容： 1，交易发送方是不是一个签名的用户 2，存证是否被别人创建过，创建过就抛出错误
			// 首先去创建签名交易，通过ensure_signed这样的system提供的版本方法来校验
			let sender = ensure_signed(origin)?;  // 存证拥有人是交易发送方，只有拥有人才可以调用存证，sender即当前交易发送方
  			// 如果存在存证，返回错误 ProofAlreadyExist
  			// ps:ensure!宏是确保表达式中的结果为true，这里取反操作
			ensure!(!Proofs::<T>::contains_key(&claim),Error::<T>::ProofAlreadyExist);  // 这里用到一个错误  ProofAlreadyExist，该错误需要在decl_error声明
			// 做insert操作，insert是key-value方式。这里的key-value是一个tuple
			// 这个tuple的第一个元素是AccountId；第二个是当前交易所处的区块，使用系统模块提供的block_number工具方法获取
			Proofs::<T>::insert(&claim,(sender.clone(),frame_system::Module::<T>::block_number()));  // 插入操作
			// 触发一个event来通知客户端，RawEvent由宏生成；   sender:存在拥有人；claim:存在hash值 通过event通知客户端
			Self::deposit_event(RawEvent::ClaimCreated(sender,claim));   // ClaimCreated事件，需要decl_event处理
			// 返回ok
			Ok(())

		}

		#[weight = 0]
		pub fn revoke_claim(origin,claim: Vec<u8>) -> dispatch::DispatchResult{
			let sender = ensure_signed(origin)?;  // 交易发送方式已签名的， 存证拥有人是交易发送方，只有拥有人才可以吊销存证

  			// 判断存储单元里面是存在这样一个存证；如果不存在，抛出错误，错误叫ClaimNotExist
			ensure!(Proofs::<T>::contains_key(&claim),Error::<T>::ClaimNotExist);

			// 获取这样的存证  owner: accountId   block_number
			let (owner,_block_number) = Proofs::<T>::get(&claim);  // 通过get api获取这样的一个存证

			ensure!(!Proofs::<T>::contains_key(&claim),Error::<T>::ProofAlreadyExist);   // 确保交易发送方是的存证人，如果不是，返回Error，这个Error叫NotClaimOwner

			// 以上校验完成之后，就可以删除的存证
		    // 存储向上调用remove函数进行删除
		    Proofs::<T>::remove(&claim);

			// 触发一个事件，返回存证人和hash
		    Self::deposit_event(RawEvent::ClaimRevoked(sender,claim));

			// 返回
			Ok(())
		}


		#[weight = 0]
		pub fn move_claim(origin,claim: Vec<u8>) -> dispatch::DispatchResult{
			let sender = ensure_signed(origin)?;  // 交易发送方式已签名的， 存证拥有人是交易发送方，只有拥有人才可以吊销存证
			ensure!(Proofs::<T>::contains_key(&claim),Error::<T>::ClaimNotExist);
			// 存储向上调用mutate 修改(id, &who, |a| a.is_frozen = true);
			Proofs::<T>::mutate(&claim,  |senders| {*senders=(sender.clone(),frame_system::Module::<T>::block_number());});
			// 触发一个事件，返回存证人和hash
		    Self::deposit_event(RawEvent::ClaimRevoked(sender,claim));

			// 返回
			Ok(())
		}

	}
}
